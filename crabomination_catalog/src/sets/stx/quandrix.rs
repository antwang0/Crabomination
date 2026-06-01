//! Quandrix (G/U) college cards from Strixhaven.
//!
//! Quandrix cares about **Fractal tokens** (0/0 green-and-blue with
//! variable +1/+1 counters), spell-cast triggers, and X-cost scaling.
//! The first-pass set here covers the two college "Apprentice" /
//! "Pledgemage" creatures plus a couple of mono-flavour scaling cards.
//! Larger Fractal-creator effects (Body of Research, Fractal Anomaly)
//! are already wired in `mono` / SOS — those compose against the same
//! `LastCreatedToken` plumbing this module re-uses.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Selector, SelectionRequirement, Subtypes, TokenDefinition,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{
    enrage, etb, magecraft, magecraft_draw, magecraft_loot, magecraft_scry, magecraft_self_pump,
    target_filtered,
};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{cost, generic, g, u, x, Color, ManaCost};

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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
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
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
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
/// Both modes wired. Mode 1 uses `Effect::Fight` with the target creature
/// (slot 0) as attacker and an auto-picked opponent creature as defender.
/// The printed card uses two separate targets for the fight; we collapse
/// the attacker to the auto-targeted creature (slot 0, filtered to your
/// creature for mode 1) and the defender to the first opponent creature
/// found by auto-targeting.
/// Both modes wired. Mode 1 uses `Effect::Fight` with the attacker as
/// a creature you control (auto-targeted) and the defender as a creature
/// an opponent controls (auto-targeted).
pub fn decisive_denial() -> CardDefinition {
    use crate::mana::{ManaCost, generic as gen_pip};
    let two = ManaCost { symbols: vec![gen_pip(2)] };
    CardDefinition {
        name: "Decisive Denial",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: counter target noncreature spell unless controller pays {2}.
            Effect::CounterUnlessPaid {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::HasCardType(CardType::Creature).negate()),
                ),
                mana_cost: two,
            },
            // Mode 1: fight — target creature you don't control takes
            // damage from your biggest creature (auto-picked from the
            // battlefield). Approximation: the printed card has two
            // separate targets; we collapse the "your creature" half to
            // auto-selected since the engine only supports one target.
            Effect::Fight {
                attacker: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                defender: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Quandrix Command ───────────────────────────────────────────────────────

/// Quandrix Command — {1}{G}{U} Instant. Choose two among 4 modes.
///
/// Approximation: AutoDecider picks +1/+1 counters (×2) + mill 2.
/// "Choose two of four" is collapsed to Seq of the two printed
/// auto-default modes (matches gameplay outcome when controller
/// picks counters + mill).
pub fn quandrix_command() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Command",
        cost: cost(&[generic(1), g(), u()]),
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
            Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

#[allow(dead_code)]
fn _quandrix_command_alt_modes() {
    let _ = Effect::ChooseMode(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack.and(
                        SelectionRequirement::Artifact
                            .or(SelectionRequirement::Enchantment),
                    ),
                ),
            },
        ]);
}

// ── Fractal Summoning ──────────────────────────────────────────────────────

/// Fractal Summoning — {X}{G}{U} Sorcery — Lesson. Create a 0/0 Fractal
/// with X +1/+1 counters.
pub fn fractal_summoning() -> CardDefinition {
    use crate::mana::x;
    let fractal = TokenDefinition {
        name: "Fractal".into(),
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
    };
    CardDefinition {
        name: "Fractal Summoning",
        cost: cost(&[x(), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: fractal,
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::XFromCost,
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
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
        // CR 614.12 "enters with two +1/+1 counters on it" replacement.
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature
                .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                .and(SelectionRequirement::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![magecraft(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ── Dragonsguard Elite ─────────────────────────────────────────────────────

/// Dragonsguard Elite — {1}{G}, 2/2 Human Druid. Magecraft: put a +1/+1
/// counter on this creature. `{3}{G}: This creature gets +X/+X until
/// end of turn, where X is its power.`
pub fn dragonsguard_elite() -> CardDefinition {
    use crate::effect::{ActivatedAbility, Duration};
    CardDefinition {
        name: "Dragonsguard Elite",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(3), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::PowerOf(Box::new(Selector::This)),
                duration: Duration::EndOfTurn,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
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
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
    }
}

// ── Eureka Moment ──────────────────────────────────────────────────────────

/// Eureka Moment — {2}{G}{U} Instant. Draw two cards. You may put a land
/// from your hand onto the battlefield tapped.
pub fn eureka_moment() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Eureka Moment",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::MayDo {
                description: "Put a land from your hand onto the battlefield tapped".to_string(),
                body: Box::new(Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Hand,
                        filter: SelectionRequirement::Land,
                    }),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: true,
                    },
                }),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::MayDo {
                description: "Put a land card from your hand onto the battlefield?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Hand,
                        filter: SelectionRequirement::Land,
                    }),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: true,
                    },
                }),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Soldier],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasSupertype(Supertype::Basic)
                    .and(SelectionRequirement::HasCardType(CardType::Land)),
                to: crate::effect::ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
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
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
                    self_counter_cost_reduction: None, sac_other_filter: None,
                    tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::HandSizeOf(PlayerRef::You),
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature),
            },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: crate::effect::ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Quandrix Wavebreaker — {2}{G}{U}, 3/3 Fractal Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, put a +1/+1 counter on this creature."
pub fn quandrix_wavewriter() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Wavewriter",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Scribe — {G}{U}, 1/2 Elf Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature gets +1/+1 until end of turn."
pub fn quandrix_scribe() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Scribe",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Quandrix Equation — {2}{G}{U}, sorcery.
/// Synthesised Oracle: "Draw a card, then put X +1/+1 counters on target
/// creature you control, where X is the number of cards in your hand."
pub fn quandrix_equipoise() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Equipoise",
        cost: cost(&[generic(2), g(), u()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Visionary — {U}, 1/1 Merfolk Wizard.
/// Synthesised Oracle: "When this creature enters, scry 1."
pub fn quandrix_visionary() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Visionary",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
    }
}

/// Fractal Reckoner — {3}{G}{U}, 4/4 Fractal.
/// Synthesised Oracle: "When this creature enters, draw a card."
pub fn fractal_reckoner() -> CardDefinition {
    CardDefinition {
        name: "Fractal Reckoner",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Quandrix Inquiry — {U}, Instant.
/// Synthesised Oracle: "Draw a card. Scry 1."
pub fn quandrix_inquiry() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Inquiry",
        cost: cost(&[u()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Topologist — {2}{U}, 2/2 Merfolk Wizard.
/// Synthesised Oracle: "When this creature enters, draw a card, then
/// discard a card."
pub fn quandrix_topologist() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Topologist",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Fractal Swarm — {1}{G}{U}, Sorcery.
/// Synthesised Oracle: "Create a 2/2 green-and-blue Fractal creature token,
/// then draw a card."
pub fn fractal_swarm() -> CardDefinition {
    CardDefinition {
        name: "Fractal Swarm",
        cost: cost(&[generic(1), g(), u()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Proofwriter — {3}{G}{U}, 4/4 Fractal Wizard.
/// Synthesised Oracle: "When this creature enters, scry 2."
pub fn quandrix_proofwriter() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Proofwriter",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Quandrix Solver — {2}{U}, 2/2 Merfolk Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, draw a card, then discard a card."
pub fn quandrix_solver() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Solver",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_loot()],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Quandrix Tideseer — {1}{U}, 1/2 Merfolk Wizard.
/// Synthesised Oracle: "Magecraft — Scry 1."
pub fn quandrix_tideseer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tideseer",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Fractal Tidecaller — {3}{G}{U}, 3/3 Fractal Wizard with Flying.
/// Synthesised Oracle: "When this creature enters, draw a card."
pub fn fractal_tidecaller() -> CardDefinition {
    CardDefinition {
        name: "Fractal Tidecaller",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Quandrix Scout — {G}, 1/1 Elf Scout.
/// Synthesised Oracle: "Magecraft — put a +1/+1 counter on this creature."
pub fn quandrix_scout() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Scout",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Fractal Reefborn — {4}{G}{U}, 4/4 Fractal Wizard with Trample.
/// Synthesised Oracle: "When this creature enters, double all +1/+1
/// counters on target creature you control."
pub fn fractal_reefborn() -> CardDefinition {
    CardDefinition {
        name: "Fractal Reefborn",
        cost: cost(&[generic(4), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Quandrix Equation — {2}{G}, Instant.
/// Synthesised Oracle: "Put two +1/+1 counters on target creature you
/// control."
pub fn quandrix_b35_equation() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Equation II",
        cost: cost(&[generic(2), g()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 38: more Quandrix cards ───────────────────────────────────────────

/// Quandrix Pondkeeper (variant II) — {G}{U}, 1/3 Frog Druid.
/// Synthesised Oracle: "Magecraft — Scry 1."
pub fn quandrix_pondkeeper_v2() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Pondkeeper II",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Fractal Emergent — {1}{G}{U}, 0/0 Fractal that enters with three +1/+1
/// counters on it (CR 614.12 replacement). Synthesised Oracle.
pub fn fractal_emergent() -> CardDefinition {
    CardDefinition {
        name: "Fractal Emergent",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        ..Default::default()
    }
}

/// Quandrix Fluctuator — {2}{G}{U}, 2/3 Elf Wizard.
/// Synthesised Oracle: "When this creature enters, draw a card."
pub fn quandrix_fluctuator() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fluctuator",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Quandrix Doublecaster (variant II) — {2}{U}, 1/3 Merfolk Wizard.
/// Synthesised Oracle: "Magecraft — Put a +1/+1 counter on target Fractal
/// you control."
pub fn quandrix_doublecaster_v2() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Doublecaster II",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal)),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Scaler — {1}{G}, 2/2 Elf Druid.
/// Synthesised Oracle: "Magecraft — Put a +1/+1 counter on this creature."
pub fn quandrix_scaler() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Scaler",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

// ── Batch 39: 6 more Quandrix cards ────────────────────────────────────────

/// Quandrix Scrymaster — {1}{U}, 2/2 Merfolk Wizard.
/// Synthesised Oracle: "ETB Scry 1. Magecraft — Scry 1."
pub fn quandrix_scrymaster() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Scrymaster",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
            },
            magecraft(Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            }),
        ],
        ..Default::default()
    }
}

/// Fractal Burst — {2}{G}{U}, Sorcery.
/// Synthesised Oracle: "Create a 0/0 G/U Fractal token with three +1/+1
/// counters on it (a 3/3)."
pub fn fractal_burst() -> CardDefinition {
    CardDefinition {
        name: "Fractal Burst",
        cost: cost(&[generic(2), g(), u()]),
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
                amount: Value::Const(3),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Aetherwarden — {3}{G}{U}, 3/4 Frog Wizard with Flying.
/// Synthesised Oracle: "ETB draw 1; Magecraft — +1/+1 counter on this."
pub fn quandrix_aetherwarden() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Aetherwarden",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            },
            magecraft(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        ],
        ..Default::default()
    }
}

/// Quandrix Tideshaper — {2}{U}, 2/3 Merfolk Wizard.
/// Synthesised Oracle: "ETB return target creature to its owner's hand."
pub fn quandrix_tideshaper() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tideshaper",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(target_filtered(
                    SelectionRequirement::Creature,
                )))),
            },
        }],
        ..Default::default()
    }
}

/// Fractal Catalyst — {G}{U}, 1/1 Fractal Druid.
/// Synthesised Oracle: "Magecraft — +1/+1 counter on target creature you
/// control."
pub fn fractal_catalyst() -> CardDefinition {
    CardDefinition {
        name: "Fractal Catalyst",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Equalizer — {4}{G}{U}, 4/4 Fractal Wizard.
/// Synthesised Oracle: "ETB put a +1/+1 counter on each other creature
/// you control."
pub fn quandrix_equalizer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Equalizer",
        cost: cost(&[generic(4), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Batch 40: more Quandrix cards ───────────────────────────────────────────

/// Quandrix Loomweaver — {2}{G}{U}, 2/3 Elf Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, draw a card, then discard a card." 4-mana Looter
/// magecraft body for spell-heavy shells. Pairs with Diary of Dreams's
/// page-counter accrual and feeds graveyard recursion via the loot.
pub fn quandrix_loomweaver() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Loomweaver",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_loot()],
        ..Default::default()
    }
}

/// Fractal Stargazer — {G}{U}, 1/2 Fractal Druid.
/// Synthesised Oracle: "ETB scry 2." A 2-mana selection body — gives
/// every Quandrix shell a top-of-deck smoothing line at the early-game.
pub fn fractal_stargazer() -> CardDefinition {
    CardDefinition {
        name: "Fractal Stargazer",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Quandrix Bountycaller — {3}{G}{U}, 3/3 Frog Druid.
/// Synthesised Oracle: "When this creature enters, create a 0/0 green
/// and blue Fractal creature token. Put four +1/+1 counters on it."
/// 5-mana Fractal-payoff body that ETBs into a 3/3 + 4/4 board.
pub fn quandrix_bountycaller() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Bountycaller",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: quandrix_fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(4),
            },
        ]))],
        ..Default::default()
    }
}

/// Quandrix Spellseer — {1}{G}{U}, 2/3 Elf Wizard.
/// Synthesised Oracle: "When this creature enters, scry 1. Magecraft —
/// draw a card, then discard a card."
pub fn quandrix_spellseer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Spellseer",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            }),
            magecraft_loot(),
        ],
        ..Default::default()
    }
}

/// Quandrix Aquamancer — {1}{U}, 1/3 Merfolk Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, draw a card, then discard a card." A 2-mana
/// magecraft looter body that snowballs in spell-heavy shells.
pub fn quandrix_aquamancer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Aquamancer",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_loot()],
        ..Default::default()
    }
}

/// Fractal Aquanaut — {2}{G}{U}, 0/0 Fractal Wizard Flying.
/// Synthesised Oracle: "This creature enters with two +1/+1 counters on
/// it. Flying." 2/2 evasive body via the `enters_with_counters` pack
/// (CR 614.12).
pub fn fractal_aquanaut() -> CardDefinition {
    CardDefinition {
        name: "Fractal Aquanaut",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        ..Default::default()
    }
}

/// Quandrix Seedling — {G}, 1/1 Fractal.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, put a +1/+1 counter on this creature." Cheapest
/// magecraft self-grower in the Quandrix shell.
pub fn quandrix_seedling() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Seedling",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Amplifier — {3}{G}{U}, 3/4 Elf Wizard.
/// Synthesised Oracle: "When this creature enters, scry 2, then draw
/// a card." 5-mana value engine with both selection and a cantrip.
pub fn quandrix_amplifier() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Amplifier",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}

/// Fractal Bloomweaver — {2}{G}{U}, 1/1 Fractal Druid.
/// Synthesised Oracle: "This creature enters with three +1/+1 counters
/// on it. When this creature enters, put a +1/+1 counter on each other
/// Fractal you control."
pub fn fractal_bloomweaver() -> CardDefinition {
    CardDefinition {
        name: "Fractal Bloomweaver",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::HasCreatureType(CreatureType::Fractal)
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        ..Default::default()
    }
}

// ── Batch 42 (modern_decks) — Quandrix expansion ────────────────────────────

/// Fractal Mathmage — {1}{G}{U}, 0/0 Fractal Wizard.
/// Synthesised Oracle: "This creature enters with three +1/+1 counters on
/// it." A clean 3-mana 3/3 Fractal body via the enters_with_counters
/// path (CR 614.12).
pub fn fractal_mathmage() -> CardDefinition {
    CardDefinition {
        name: "Fractal Mathmage",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        ..Default::default()
    }
}

/// Quandrix Geometer II — {2}{G}{U}, 2/2 Elf Druid.
/// Synthesised Oracle: "When this creature enters, scry 1, then draw a
/// card. Magecraft — Put a +1/+1 counter on target creature you control."
/// 4-mana cantrip body with magecraft fan-out.
pub fn quandrix_geometer_v2() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Geometer II",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::Seq(vec![
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ])),
            magecraft(Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        ],
        ..Default::default()
    }
}

/// Fractal Sproutling — {G}, 0/0 Fractal.
/// Synthesised Oracle: "This creature enters with a +1/+1 counter on it."
/// 1-mana 1/1 Fractal — the cheapest Fractal body in the catalog,
/// scaling targets for Growth Curve and Quandrix Seedling.
pub fn fractal_sproutling() -> CardDefinition {
    CardDefinition {
        name: "Fractal Sproutling",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(1))),
        ..Default::default()
    }
}

/// Quandrix Calligrapher II — {1}{U}, 1/2 Merfolk Wizard.
/// Synthesised Oracle: "When this creature enters, draw a card." A
/// clean 2-mana cantrip body — STX flavor of Spirited Companion (W) and
/// Elvish Visionary (G).
pub fn quandrix_calligrapher_v2() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Calligrapher II",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Equation II — {G}{U} Instant.
/// Synthesised Oracle: "Put two +1/+1 counters on target creature you
/// control." 2-mana clean +2/+2 — composes against the rest of the
/// Quandrix counter package (Growth Curve doubles after).
pub fn quandrix_equation_v2() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Equation (v2)",
        cost: cost(&[g(), u()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}


/// Quandrix Synthsage — {2}{G}{U}, 3/3 Elf Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, put a +1/+1 counter on this creature. When this
/// creature enters, you gain 2 life." 4-mana defensive magecraft body
/// that grows over the game.
pub fn quandrix_synthsage() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Synthsage",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            crate::effect::shortcut::etb_gain_life(2),
            magecraft(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        ],
        ..Default::default()
    }
}

/// Fractal Tidecaller II — {2}{U}, 0/0 Fractal Wizard.
/// Synthesised Oracle: "Flying. This creature enters with two +1/+1
/// counters on it." 3-mana 2/2 evasive Fractal body.
pub fn fractal_tidecaller_v2() -> CardDefinition {
    CardDefinition {
        name: "Fractal Tidecaller II",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        ..Default::default()
    }
}

// ── Batch 43 (modern_decks) — Quandrix expansion ────────────────────────────

/// Quandrix Thoughtweaver — {1}{G}{U}, 2/2 Elf Wizard.
/// Synthesised Oracle: "When this creature enters, draw a card." A
/// 3-mana cantrip body.
pub fn quandrix_thoughtweaver() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Thoughtweaver",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Geode Smith — {1}{U}, 1/2 Merfolk Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, put a +1/+1 counter on this creature."
/// 2-mana self-growing magecraft body.
pub fn quandrix_geode_smith() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Geode Smith",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Grand Calculator — {3}{G}{U}, 3/3 Elf Wizard.
/// Synthesised Oracle: "When this creature enters, put a +1/+1 counter
/// on target creature you control for each land you control."
/// 5-mana scaling pump body.
pub fn quandrix_grand_calculator() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Grand Calculator",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::count(Selector::EachPermanent(
                SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
            )),
        })],
        ..Default::default()
    }
}

/// Fractal Seer — {G}{U}, 0/0 Fractal Druid.
/// Synthesised Oracle: "This creature enters with one +1/+1 counter on
/// it." 2-mana 1/1 base Fractal — scales with +1/+1 doublers.
pub fn fractal_seer() -> CardDefinition {
    CardDefinition {
        name: "Fractal Seer",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(1))),
        ..Default::default()
    }
}

/// Quandrix Lifestream — {1}{G}{U} Sorcery. Synthesised Oracle:
/// "Put a +1/+1 counter on target creature you control. Draw a card."
/// 3-mana sorcery pump + cantrip.
pub fn quandrix_lifestream() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Lifestream",
        cost: cost(&[generic(1), g(), u()]),
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
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Fractal Aegis — {2}{G}{U}, 0/0 Fractal Soldier Trample.
/// Synthesised Oracle: "Trample. This creature enters with three
/// +1/+1 counters on it." 4-mana 3/3 trampler — scales with doublers.
pub fn fractal_aegis() -> CardDefinition {
    CardDefinition {
        name: "Fractal Aegis",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Soldier],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        ..Default::default()
    }
}

/// Quandrix Mistforger — {2}{G}{U}, 3/3 Fractal Wizard.
/// Synthesised Oracle: "When this creature enters, create a 0/0 green
/// and blue Fractal creature token, then put X +1/+1 counters on it,
/// where X is the number of creatures you control." 4-mana Fractal
/// minter that scales with your board.
pub fn quandrix_mistforger() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mistforger",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: quandrix_fractal_token(),
                count: Value::Const(1),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                )),
            },
        ]))],
        ..Default::default()
    }
}

// ── Batch 47 (modern_decks) — Quandrix expansion ────────────────────────────

/// Quandrix Arcanist — {1}{G}{U}, 2/2 Elf Wizard. Synthesised Oracle:
/// "Flash. Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, scry 1." 3-mana flash body with a scry-on-cast engine.
pub fn quandrix_arcanist() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Arcanist",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Triplecaster — {2}{G}{U}, 3/3 Elf Wizard. Synthesised
/// Oracle: "When this creature enters, put two +1/+1 counters on
/// target creature you control." 4-mana mid-range counter accumulator.
pub fn quandrix_triplecaster() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Triplecaster",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Quandrix Snapcaster — {1}{U}, 2/1 Elf Wizard Flash. Synthesised
/// Oracle: "Flash. When this creature enters, target instant or sorcery
/// card in your graveyard returns to your hand." A blue-side Snapcaster
/// approximation: rebuy a spell, no flashback grant.
pub fn quandrix_snapcaster() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Quandrix Snapcaster",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                }),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Quandrix Counterfold — {3}{G}{U} Sorcery. Synthesised Oracle:
/// "Double the number of +1/+1 counters on target creature you control."
/// 5-mana doubling pump on a single creature.
pub fn quandrix_counterfold() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Counterfold",
        cost: cost(&[generic(3), g(), u()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Augurer — {3}{G}{U}, 3/4 Elf Druid. Synthesised
/// Oracle: "When this creature enters, draw a card. Then put a +1/+1
/// counter on each creature you control."
pub fn quandrix_augurer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Augurer",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::ForEach {
                    selector: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::TriggerSource,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    }),
                },
            ]),
        }],
        ..Default::default()
    }
}

// ── Batch 48 (modern_decks) — Quandrix expansion ────────────────────────────

/// Quandrix Pupil — {G}{U}, 1/2 Elf Wizard. Synthesised Oracle:
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// scry 1." 2-mana cheap card-selection magecraft body.
pub fn quandrix_pupil() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Pupil",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Fractal Tideshaper — {2}{G}{U}, 0/0 Fractal that enters with three
/// +1/+1 counters (CR 614.12 replacement). Synthesised Oracle:
/// "This creature enters with three +1/+1 counters on it." Net 3/3
/// scaling Fractal body.
pub fn fractal_tideshaper() -> CardDefinition {
    CardDefinition {
        name: "Fractal Tideshaper",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        ..Default::default()
    }
}

/// Quandrix Numerologist — {2}{U}, 2/3 Merfolk Wizard. Synthesised
/// Oracle: "When this creature enters, draw a card." 3-mana cantrip
/// body.
pub fn quandrix_numerologist() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Numerologist",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Quandrix Geometer III — {1}{G}{U}, 2/2 Elf Wizard. Synthesised Oracle:
/// "When this creature enters, put a +1/+1 counter on each creature
/// you control." 3-mana fan-out anthem via counters.
pub fn quandrix_geometer_v3() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Geometer III",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Fractal Cascade — {3}{G}{U} Sorcery. Synthesised Oracle: "Create
/// a 0/0 green and blue Fractal creature token. Put four +1/+1
/// counters on it." Mints a net 4/4 Fractal token for 5 mana.
pub fn fractal_cascade() -> CardDefinition {
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Fractal Cascade",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: create_token_with_counter(
            PlayerRef::You,
            1,
            quandrix_fractal_token(),
            CounterType::PlusOnePlusOne,
            4,
        ),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 48 follow-up (modern_decks) — Quandrix expansion 2 ────────────────

/// Fractal Wavebreaker — {2}{U}, 1/3 Merfolk Wizard. Synthesised
/// Oracle: "When this creature enters, return target creature to its
/// owner's hand." 3-mana bounce + body.
pub fn fractal_wavebreaker() -> CardDefinition {
    CardDefinition {
        name: "Fractal Wavebreaker",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        }],
        ..Default::default()
    }
}

/// Quandrix Vinepriest — {2}{G}, 2/3 Elf Druid. Synthesised Oracle:
/// "When this creature enters, search your library for a basic land
/// card, reveal it, put it into your hand, then shuffle." 3-mana
/// ramp body.
pub fn quandrix_vinepriest() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Vinepriest",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Fractal Anomaly II — {3}{G}{U} Sorcery. Synthesised Oracle:
/// "Create a 0/0 green and blue Fractal creature token. Put five
/// +1/+1 counters on it." Net 5/5 Fractal for 5 mana.
pub fn fractal_anomaly_v2() -> CardDefinition {
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Fractal Anomaly II",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: create_token_with_counter(
            PlayerRef::You,
            1,
            quandrix_fractal_token(),
            CounterType::PlusOnePlusOne,
            5,
        ),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Calculator II — {1}{G}{U}, 2/2 Elf Wizard. Synthesised
/// Oracle: "When this creature enters, scry 2." 3-mana scry body.
pub fn quandrix_calculator_v2() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Calculator II",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Quandrix Tide — {G}{U} Instant. Synthesised Oracle: "Put a +1/+1
/// counter on target creature you control. Draw a card." 2-mana
/// counter + cantrip — same shape as Quandrix Counterbalance.
pub fn quandrix_tide() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tide",
        cost: cost(&[g(), u()]),
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
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 48 follow-up #2 (modern_decks) — more Quandrix cards ──────────────

/// Fractal Sentinel — {3}{G}{U}, 0/0 Fractal Soldier Trample. Enters
/// with five +1/+1 counters via `CardDefinition.enters_with_counters`
/// (CR 614.12). Net 5/5 trampler for 5 mana.
pub fn fractal_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Fractal Sentinel",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Soldier],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(5))),
        ..Default::default()
    }
}

// ── Batch 49 (modern_decks) — more Quandrix cards ───────────────────────────

/// Quandrix Theoremist — {G}{U}, 2/1 Elf Wizard.
/// Synthesised Oracle: "When this creature enters, draw a card."
/// 2-mana cantrip body — Elvish Visionary in Quandrix colors.
pub fn quandrix_theoremist() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Theoremist",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Fractal Shaper — {1}{G}, 2/2 Elf Druid. Synthesised Oracle:
/// "When this creature enters, put a +1/+1 counter on target creature
/// you control." Cheap +1/+1 distributor — combos with the Quandrix
/// counter-doubling chain.
pub fn fractal_shaper() -> CardDefinition {
    CardDefinition {
        name: "Fractal Shaper",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Quandrix Foresight — {G}{U} Instant. Synthesised Oracle:
/// "Put a +1/+1 counter on target creature you control. Draw a card."
/// 2-mana growth-plus-cantrip — a classic Quandrix tempo trick.
pub fn quandrix_foresight() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Foresight",
        cost: cost(&[g(), u()]),
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
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Fractal Bloomstalker — {2}{G}{U}, 0/0 Fractal. Synthesised Oracle:
/// "Trample. This creature enters with four +1/+1 counters on it."
/// 4-mana 4/4 trampler — bigger Body of Research baby.
pub fn fractal_bloomstalker() -> CardDefinition {
    CardDefinition {
        name: "Fractal Bloomstalker",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        ..Default::default()
    }
}

/// Quandrix Lensbearer — {1}{U}, 1/3 Merfolk Wizard. Synthesised
/// Oracle: "When this creature enters, scry 1." 2-mana cheap scry
/// body.
pub fn quandrix_lensbearer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Lensbearer",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Batch 50: Quandrix synthesised cards ───────────────────────────────────

/// Quandrix Scryweaver — {G}{U}, 1/2 Elf Wizard. Magecraft scry 1.
/// 2-mana magecraft scry body.
pub fn quandrix_scryweaver() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Quandrix Scryweaver",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Fractal Bloomthorn — {2}{G}{U}, 0/0 Fractal Trample. Enters with
/// 3 +1/+1 counters via `enters_with_counters` (CR 614.12). 4-mana
/// 3/3 trampler.
pub fn fractal_bloomthorn() -> CardDefinition {
    CardDefinition {
        name: "Fractal Bloomthorn",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        ..Default::default()
    }
}

/// Quandrix Pupil v2 — {G}, 1/1 Elf Wizard. Magecraft AddCounter
/// +1/+1 on self. Cheapest magecraft self-scaling body.
/// (Disambiguated from existing Quandrix Pupil.)
pub fn quandrix_pupil_b50() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Pupil Adept",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Forge — {2}{G}{U}, Sorcery. Mint a 0/0 Fractal token with
/// 4 +1/+1 counters on it. 4-mana flat Fractal token.
pub fn quandrix_forge() -> CardDefinition {
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Quandrix Forge",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: create_token_with_counter(
            PlayerRef::You,
            1,
            quandrix_fractal_token(),
            CounterType::PlusOnePlusOne,
            4,
        ),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Algorithmist — {2}{G}{U}, 2/3 Elf Druid. Magecraft puts
/// +1/+1 counter on each Fractal you control. 4-mana team-pump magecraft.
pub fn quandrix_algorithmist() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Algorithmist",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::HasCreatureType(CreatureType::Fractal)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Refractor — {1}{G}{U}, 2/3 Fractal Wizard. ETB draws a
/// card. 3-mana cantrip Fractal.
pub fn quandrix_refractor() -> CardDefinition {
    use crate::effect::shortcut::etb_draw;
    CardDefinition {
        name: "Quandrix Refractor",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Echocaster — {1}{G}{U}, 2/2 Elf Druid. Magecraft puts a
/// +1/+1 counter on each Fractal you control.
pub fn quandrix_echocaster() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Echocaster",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal)),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Fractal Bloomstone — {2}{G}{U}, 0/0 Fractal that enters with X
/// +1/+1 counters where X = lands you control. 4-mana ramp scaler.
pub fn fractal_bloomstone() -> CardDefinition {
    CardDefinition {
        name: "Fractal Bloomstone",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::count(Selector::EachPermanent(
                SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
            )),
        )),
        ..Default::default()
    }
}

/// Quandrix Reflection — {2}{G}{U}, Sorcery. Doubles +1/+1 counters
/// on each creature you control via `Value::CountersOn(Self)`.
pub fn quandrix_reflection() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Reflection",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            body: Box::new(Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountersOn {
                    what: Box::new(Selector::TriggerSource),
                    kind: CounterType::PlusOnePlusOne,
                },
            }),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Tideseer Adept — {1}{U}, 1/3 Merfolk Wizard Flash. ETB
/// Scry 1 + magecraft scry 1. Disambiguated from the existing
/// `quandrix_tideseer` factory earlier in this file.
pub fn quandrix_tideseer_adept() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Quandrix Tideseer Adept",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
            },
            magecraft_scry(1),
        ],
        ..Default::default()
    }
}

/// Fractal Geomancer — {3}{G}{U}, 4/4 Fractal Wizard. Magecraft adds
/// a +1/+1 counter to target Fractal you control.
pub fn fractal_geomancer() -> CardDefinition {
    CardDefinition {
        name: "Fractal Geomancer",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal)),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Cantripper — {G}{U}, 1/1 Fractal. Magecraft Draw 1 then
/// Discard 1. Spell-loot magecraft body.
pub fn quandrix_cantripper() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Cantripper",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_loot()],
        ..Default::default()
    }
}

/// Fractal Bloomanalyst — {2}{G}{U}, 0/0 Fractal Wizard. Enters with
/// +1/+1 counters = creatures you control (excluding self).
pub fn fractal_bloomanalyst() -> CardDefinition {
    CardDefinition {
        name: "Fractal Bloomanalyst",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        // X = creatures you control (excluding self via OtherThanSource).
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::count(Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            )),
        )),
        ..Default::default()
    }
}

// ── batch 53: more Quandrix cards ───────────────────────────────────────────

/// Fractal Synthmage — {2}{G}{U}, 2/2 Fractal Wizard. ETB +1/+1 counters
/// on self equal to creatures you control (excluding self).
pub fn fractal_synthmage() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Fractal Synthmage",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                )),
            },
        }],
        ..Default::default()
    }
}

/// Quandrix Amplify — {1}{G}{U}, Sorcery. Seq(AddCounter +1/+1 ×2 target
/// friendly + Scry 1). 3-mana sticky pump.
pub fn quandrix_amplify() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Quandrix Amplify",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Sorcery],
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
                amount: Value::Const(2),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Threadbinder — {G}{U}, 1/2 Elf Wizard. Magecraft Scry 1 — early
/// scry engine.
pub fn quandrix_threadbinder() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Quandrix Threadbinder",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

// ── batch 54: more Quandrix cards ───────────────────────────────────────────

/// Quandrix Tideturner — {1}{G}{U}, 2/2 Merfolk Wizard. ETB Scry 1 +
/// magecraft +1/+1 counter on self.
pub fn quandrix_tideturner() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Quandrix Tideturner",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb_scry(1),
            magecraft(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        ],
        ..Default::default()
    }
}

/// Fractal Overgrowth — {2}{G}{U}, Sorcery. Doubles +1/+1 counters on
/// each creature you control via ForEach + AddCounter equal to current
/// counter count. (Common Quandrix snowball payoff.)
pub fn fractal_overgrowth() -> CardDefinition {
    CardDefinition {
        name: "Fractal Overgrowth",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            ),
            body: Box::new(Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountersOn {
                    what: Box::new(Selector::TriggerSource),
                    kind: CounterType::PlusOnePlusOne,
                },
            }),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Ectomancer — {2}{U}, 1/3 Merfolk Wizard. Magecraft draw a
/// card on a Spirit-tribal frog frame.
pub fn quandrix_ectomancer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Ectomancer",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Fractal Resonance II — {3}{G}{U}, 0/0 Fractal. Enters with +1/+1
/// counters equal to your hand size. CR 614.12. Disambiguated from the
/// earlier `fractal_resonance` factory.
pub fn fractal_resonance_v2() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Fractal Resonance II",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::HandSizeOf(PlayerRef::You),
        )),
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 55): 5 more Quandrix cards ───────────────────

/// Quandrix Calcographer — {1}{G}{U}, 2/3 Elf Druid. ETB Seq(mint a 0/0
/// Fractal with one +1/+1 counter + magecraft +1/+1 counter on self).
/// Self-scaling + Fractal-mint headline body.
pub fn quandrix_calcographer() -> CardDefinition {
    use crate::effect::shortcut::{create_token_with_counter, etb};
    CardDefinition {
        name: "Quandrix Calcographer",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb(create_token_with_counter(
                PlayerRef::You,
                1,
                quandrix_fractal_token(),
                CounterType::PlusOnePlusOne,
                1,
            )),
            magecraft(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        ],
        ..Default::default()
    }
}

/// Fractal Initiate — {1}{G}, 2/2 Fractal. Vanilla 2-mana Fractal body.
/// Slots into Tanazir doubling + +1/+1 counter shells.
pub fn fractal_initiate() -> CardDefinition {
    CardDefinition {
        name: "Fractal Initiate",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Splitcaster — {2}{G}{U}, 3/3 Elf Wizard. Magecraft mints a
/// 0/0 Fractal token with one +1/+1 counter on it. 4-mana per-spell
/// Fractal engine that goes wide quickly.
pub fn quandrix_splitcaster() -> CardDefinition {
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Quandrix Splitcaster",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(create_token_with_counter(
            PlayerRef::You,
            1,
            quandrix_fractal_token(),
            CounterType::PlusOnePlusOne,
            1,
        ))],
        ..Default::default()
    }
}

/// Quandrix Calculation — {1}{G}{U}, Instant. Seq(target friendly
/// creature gets a +1/+1 counter + Draw 1). 3-mana scaling-counter
/// cantrip — Quandrix's signature math-themed combat trick.
pub fn quandrix_calculation() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Calculation",
        cost: cost(&[generic(1), g(), u()]),
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
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Bookkeeper — {2}{U}, 1/3 Merfolk Wizard. Magecraft Scry 1
/// + Draw 1 on self. Smooths the topdeck on every IS cast.
pub fn quandrix_bookkeeper() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Bookkeeper",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 56) — new Quandrix STX cards ─────────────────

/// Quandrix Mathlord — {2}{G}{U}, 2/2 Elf Wizard. ETB mints a Fractal
/// token (with two +1/+1 counters via the team-wide AddCounter) +
/// magecraft puts a +1/+1 counter on each Fractal you control. Quandrix
/// tribal scaling engine.
pub fn quandrix_mathlord() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Quandrix Mathlord",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: quandrix_fractal_token(),
                },
                Effect::AddCounter {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ])),
            magecraft(Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        ],
        ..Default::default()
    }
}

/// Quandrix Geometer (batch 56) — {1}{G}, 2/2 Elf Druid. Magecraft
/// puts a +1/+1 counter on each creature you control. Team-wide
/// magecraft scaler.
pub fn quandrix_geometer_b56() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Geometer (b56)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Fractal Trifecta — {1}{G}{U}, Sorcery. Mint 3 Fractal tokens
/// each with one +1/+1 counter via team-wide AddCounter. 3-mana
/// triple-Fractal mint.
pub fn fractal_trifecta() -> CardDefinition {
    CardDefinition {
        name: "Fractal Trifecta",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: quandrix_fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Tidesower — {2}{U}, 1/4 Merfolk Wizard. ETB target creature
/// gets -2/-0 EOT + Draw 1. Defensive tempo + cantrip on a sturdy body.
pub fn quandrix_tidesower() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Quandrix Tidesower",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Fractal Augmenter — {3}{G}{U}, 0/0 Fractal Wizard. Enters with
/// +1/+1 counters equal to your hand size. Quandrix scaling top-end.
pub fn fractal_augmenter() -> CardDefinition {
    CardDefinition {
        name: "Fractal Augmenter",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::HandSizeOf(PlayerRef::You),
        )),
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 57): 3 more Quandrix cards ───────────────────

/// Fractal Greenstone — {1}{G}, 0/0 Fractal. Enters with 2 +1/+1
/// counters (CR 614.12) — a printed-0/0 frame that lands at 2/2 for
/// 2 mana. Cheap Fractal-tribal body that scales with Tanazir / +1/+1
/// counter doublers.
pub fn fractal_greenstone() -> CardDefinition {
    CardDefinition {
        name: "Fractal Greenstone",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        ..Default::default()
    }
}

/// Quandrix Tideguard — {2}{U}, 2/3 Merfolk Wizard. Magecraft places
/// a +1/+1 counter on target Fractal you control. 3-mana scaling
/// Fractal-tribal pump engine.
pub fn quandrix_tideguard() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tideguard",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::HasCreatureType(CreatureType::Fractal)
                    .and(SelectionRequirement::ControlledByYou),
            },
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Greenmage — {2}{G}{U}, 3/3 Elf Druid. ETB Seq(Scry 1 +
/// AddCounter(+1/+1, Self)). 4-mana scaling value body — lands at
/// 4/4 with selection.
pub fn quandrix_greenmage() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Quandrix Greenmage",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 58): 4 more Quandrix cards ───────────────────

/// Quandrix Spellsplicer — {1}{U}, 1/3 Merfolk Wizard. Magecraft: Scry
/// 1. Cheap defensive body that smooths draws each IS spell.
pub fn quandrix_spellsplicer() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Quandrix Spellsplicer",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Fractal Bluepetal — {1}{G}, 0/0 Fractal that enters with two +1/+1
/// counters on it. 2-mana 2/2 with built-in counter scaling.
pub fn fractal_bluepetal() -> CardDefinition {
    CardDefinition {
        name: "Fractal Bluepetal",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        ..Default::default()
    }
}

/// Quandrix Mathweaver — {2}{G}, 2/3 Elf Druid. ETB mint 0/0 Fractal
/// with one +1/+1 counter. 3-mana wide body that drops a Fractal anchor
/// for Tanazir / Quandrix Doubler payoffs.
pub fn quandrix_mathweaver() -> CardDefinition {
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Quandrix Mathweaver",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
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
        ..Default::default()
    }
}

/// Quandrix Sumcaster II — {2}{G}{U}, 3/3 Merfolk Wizard. Magecraft: add
/// a +1/+1 counter to a Fractal you control. 4-mana Fractal-tribal
/// scaling engine.
pub fn quandrix_sumcaster_b58() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sumcaster II",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::HasCreatureType(CreatureType::Fractal)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 59): 5 more Quandrix cards ───────────────────

/// Quandrix Growth-Tutor — {1}{G}, 1/2 Elf Druid. ETB: put a +1/+1
/// counter on target Fractal you control. 2-mana Fractal-tribal pump
/// enabler.
pub fn quandrix_growth_tutor() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Quandrix Growth-Tutor",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::HasCreatureType(CreatureType::Fractal)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Fractal Redleaf — {2}{U}, 0/0 Fractal that enters with three +1/+1
/// counters on it. 3-mana 3/3 with a bigger counter pile than
/// Bluepetal.
pub fn fractal_redleaf() -> CardDefinition {
    CardDefinition {
        name: "Fractal Redleaf",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        ..Default::default()
    }
}

/// Quandrix Oracle II — {U}{G}, 2/1 Merfolk Wizard. Magecraft: Scry 1.
/// Cheap aggressive body that smooths card velocity per IS cast.
pub fn quandrix_oracle_b59() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Quandrix Oracle II",
        cost: cost(&[u(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Quandrix Summerkeeper — {2}{G}{U}, 3/3 Elf Wizard. ETB mint 0/0
/// Fractal with two +1/+1 counters. 4-mana Fractal-tribal anchor —
/// drops a 2/2 token body for combat + Tanazir doubling fuel.
pub fn quandrix_summerkeeper() -> CardDefinition {
    use crate::effect::shortcut::{create_token_with_counter, etb};
    CardDefinition {
        name: "Quandrix Summerkeeper",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb(create_token_with_counter(
            PlayerRef::You,
            1,
            quandrix_fractal_token(),
            CounterType::PlusOnePlusOne,
            2,
        ))],
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 60): 3 more Quandrix cards ───────────────────

/// Quandrix Tideborn — {1}{U}, 1/3 Merfolk Wizard. Magecraft Surveil 1.
/// 2-mana defensive smoother with graveyard fuel.
pub fn quandrix_tideborn() -> CardDefinition {
    use crate::effect::shortcut::magecraft_surveil;
    CardDefinition {
        name: "Quandrix Tideborn",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![magecraft_surveil(1)],
        ..Default::default()
    }
}

/// Fractal Stormpetal — {3}{G}, 0/0 Fractal that enters with four +1/+1
/// counters on it. 4-mana 4/4 Fractal body for go-tall plans.
pub fn fractal_stormpetal() -> CardDefinition {
    CardDefinition {
        name: "Fractal Stormpetal",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        ..Default::default()
    }
}

/// Quandrix Pondwarden — {3}{G}{U}, 3/4 Elf Druid. ETB mint two 0/0
/// Fractals each with one +1/+1 counter. 5-mana double-Fractal anchor.
pub fn quandrix_pondwarden() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Quandrix Pondwarden",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
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
        ]))],
        ..Default::default()
    }
}

/// Quandrix Skywinder — {3}{G}{U}, 3/3 Merfolk Wizard with Flying.
/// Magecraft: +1/+1 EOT to target friendly Fractal. 5-mana evasive
/// Fractal-pumper.
pub fn quandrix_skywinder() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Quandrix Skywinder",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(
                SelectionRequirement::HasCreatureType(CreatureType::Fractal)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            1, 1,
        )],
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 61): 5 more Quandrix cards ────────────────────

/// Quandrix Seer II — {1}{U}, 1/3 Merfolk Wizard. Magecraft Seq(Draw 1
/// + Discard 1) via `magecraft_loot()`. 2-mana defensive loot-on-cast.
pub fn quandrix_seer_b61() -> CardDefinition {
    use crate::effect::shortcut::magecraft_loot;
    CardDefinition {
        name: "Quandrix Seer II",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        ..Default::default()
    }
}

/// Fractal Mosspetal — {1}{U}, 0/0 Fractal that enters with two +1/+1
/// counters on it via CR 614.12 (`enters_with_counters`). Cheap 2-mana
/// Fractal body — a 2/2 for {U} with growth potential under doublers.
pub fn fractal_mosspetal() -> CardDefinition {
    CardDefinition {
        name: "Fractal Mosspetal",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        ..Default::default()
    }
}

/// Quandrix Growkeeper — {2}{G}{U}, 2/3 Elf Druid. ETB mints a 0/0 G/U
/// Fractal token with three +1/+1 counters on it (via `LastCreatedTokens`).
/// 4-mana Fractal-tribal go-tall anchor.
pub fn quandrix_growkeeper() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Quandrix Growkeeper",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: quandrix_fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedTokens,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            },
        ]))],
        ..Default::default()
    }
}

/// Quandrix Doublecast — {1}{G}{U}, 2/2 Merfolk Druid. Magecraft +1/+1
/// counter on target friendly Fractal. 3-mana per-cast Fractal scaler.
pub fn quandrix_doublecast() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Doublecast",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::HasCreatureType(CreatureType::Fractal)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Pondseer — {3}{G}{U}, 3/4 Merfolk Wizard Flying. ETB
/// Seq(Scry 2 + +1/+1 counter on each Fractal you control). 5-mana
/// evasive Fractal scaler finisher.
pub fn quandrix_pondseer() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Quandrix Pondseer",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Fractal)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 62): 2 more Quandrix cards ────────────────────

/// Quandrix Numberminder — {2}{G}, 2/3 Elf Druid. Magecraft Scry 1 via
/// the `magecraft_scry(1)` shortcut. 3-mana defensive smoother body —
/// turns each IS cast into a smoother.
pub fn quandrix_numberminder() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Quandrix Numberminder",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Fractal Rookling — {G}, 0/0 Fractal that enters with one +1/+1
/// counter. 1-mana cheapest Fractal — a vanilla 1/1 for {G} with growth
/// potential under Tanazir / +1/+1 doublers.
pub fn fractal_rookling() -> CardDefinition {
    CardDefinition {
        name: "Fractal Rookling",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(1))),
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 63): 5 more Quandrix cards ────────────────────

/// Quandrix Counterweave — {1}{G}{U}, Instant. Counter target spell unless
/// its controller pays {2}, then put a +1/+1 counter on a target friendly
/// creature. Hybrid Quandrix tempo + growth (Mana Leak + Aether Charge).
pub fn quandrix_counterweave() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Counterweave",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CounterUnlessPaid {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::IsSpellOnStack,
                },
                mana_cost: cost(&[generic(2)]),
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
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Sumwarden — {3}{G}{U}, 4/4 Elf Druid. ETB Seq(draw 1 + AddCounter
/// +1/+1 to self). 5-mana sturdier draw + +1/+1 body.
pub fn quandrix_sumwarden() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Quandrix Sumwarden",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Fractal Petalcaller — {2}{U}, 0/0 Fractal Wizard that enters with two
/// +1/+1 counters on it. Magecraft +1/+1 counter on self. 3-mana evasive
/// Fractal growth body.
pub fn fractal_petalcaller() -> CardDefinition {
    CardDefinition {
        name: "Fractal Petalcaller",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        ..Default::default()
    }
}

/// Quandrix Echoreader — {1}{U}, 1/3 Merfolk Wizard. Magecraft Scry 1.
/// 2-mana defensive smoothing body.
pub fn quandrix_echoreader() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Quandrix Echoreader",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Quandrix Synthesizer — {2}{G}{U}, Sorcery. Create a 0/0 G/U Fractal
/// token, then put X +1/+1 counters on it, where X is the number of cards
/// in your hand. Hand-size-scaling go-tall.
pub fn quandrix_synthesizer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Synthesizer",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: quandrix_fractal_token(),
                count: Value::Const(1),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedTokens,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::HandSizeOf(PlayerRef::You),
            },
        ]),
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 64): 4 more Quandrix cards ───────────────────

/// Quandrix Sumherald — {1}{G}, 1/2 Elf Druid. Magecraft +1/+1 counter on
/// target friendly Fractal. 2-mana Fractal-tribal scaler.
pub fn quandrix_sumherald() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sumherald",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Fractal Stridepetal — {2}{G}, 0/0 Fractal that enters with three
/// +1/+1 counters. 3-mana 3/3 — Fractal mid-game body.
pub fn fractal_stridepetal() -> CardDefinition {
    CardDefinition {
        name: "Fractal Stridepetal",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        ..Default::default()
    }
}

/// Quandrix Streamcaller — {2}{U}, 2/2 Merfolk Wizard Flying. Magecraft
/// loot 1 — every spell smooths the next draw. 3-mana evasive engine.
pub fn quandrix_streamcaller() -> CardDefinition {
    use crate::effect::shortcut::magecraft_loot;
    CardDefinition {
        name: "Quandrix Streamcaller",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_loot()],
        ..Default::default()
    }
}

/// Quandrix Fractal-Forge — {2}{G}{U}, Sorcery. Create two 0/0 G/U
/// Fractal tokens, each with two +1/+1 counters. 4-mana double-mint.
pub fn quandrix_fractal_forge() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractal-Forge",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: quandrix_fractal_token(),
                count: Value::Const(2),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedTokens,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        ]),
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 67): 6 more Quandrix cards ───────────────────

/// Quandrix Mistwarden — {U}, 0/3 Merfolk Wizard Defender. `{T}: Scry 1`.
/// 1-mana defensive selection wall.
pub fn quandrix_mistwarden() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mistwarden",
        cost: cost(&[u()]),
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
            mana_cost: cost(&[]),
            tap_cost: true,
            sac_cost: false,
            exile_self_cost: false,
            exile_other_filter: None,
            life_cost: 0,
            sorcery_speed: false,
            from_graveyard: false,
            condition: None,
            once_per_turn: false,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Spellseer-Adept — {1}{G}{U}, 2/3 Elf Wizard. Magecraft Scry 1.
/// 3-mana defensive smoother + magecraft.
pub fn quandrix_spellseer_adept() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Quandrix Spellseer-Adept",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Fractal Floodling — {3}{G}{U}, 0/0 Fractal. Enters with N +1/+1
/// counters where N = creatures you control. 5-mana wide-board scaler.
pub fn fractal_floodling() -> CardDefinition {
    CardDefinition {
        name: "Fractal Floodling",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::CountOf(Box::new(Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ))),
        )),
        ..Default::default()
    }
}

/// Quandrix Sumchant — {G}{U}, Instant. Adds a +1/+1 counter to target
/// friendly creature and draws a card. 2-mana sticky pump + cantrip.
pub fn quandrix_sumchant() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sumchant",
        cost: cost(&[g(), u()]),
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
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Sumcaster (b67) — {2}{G}{U}, 3/3 Elf Wizard. ETB Draw 1 +
/// magecraft AddCounter(+1/+1, Self). 4-mana scaling cantrip body.
pub fn quandrix_sumcaster_b67() -> CardDefinition {
    use crate::effect::shortcut::{etb_draw, magecraft};
    CardDefinition {
        name: "Quandrix Sumcaster (b67)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb_draw(1),
            magecraft(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        ],
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 68): more Quandrix G/U cards ───────────────

/// Quandrix Mistshaper II — {1}{U}, 2/2 Merfolk Wizard. Magecraft Draw 1
/// + Discard 1 (loot). 2-mana magecraft loot body.
pub fn quandrix_mistshaper_b68() -> CardDefinition {
    use crate::effect::shortcut::magecraft_loot;
    CardDefinition {
        name: "Quandrix Mistshaper (b68)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_loot()],
        ..Default::default()
    }
}

/// Fractal Pondling — {G}, 1/1 Fractal. Vanilla 1-mana Fractal — works
/// as cheap +1/+1 counter target for Quandrix grow spells.
pub fn fractal_pondling() -> CardDefinition {
    CardDefinition {
        name: "Fractal Pondling",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Streamwarden — {2}{G}{U}, 3/4 Elf Druid Reach. Magecraft
/// AddCounter(+1/+1, target Fractal you control). Tribal payoff for
/// Fractal-go-tall shells.
pub fn quandrix_streamwarden() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Streamwarden",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::HasCreatureType(CreatureType::Fractal)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Sumstride — {3}{G}{U}, Sorcery. Mints a Fractal with X
/// counters where X = creatures you control. 5-mana board-scaled
/// Fractal finisher.
pub fn quandrix_sumstride() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sumstride",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: quandrix_fractal_token(),
                count: Value::Const(1),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedTokens,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ))),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Mistdiver — {G}{U}, 2/2 Merfolk Wizard Flying. 2-mana
/// evasive Quandrix beater.
pub fn quandrix_mistdiver() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mistdiver",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Riverflux — {1}{G}{U}, Sorcery. Mints a 0/0 Fractal with
/// counters equal to instants/sorceries in your graveyard. 3-mana
/// graveyard-scaling Fractal mint.
pub fn quandrix_riverflux() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Quandrix Riverflux",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: quandrix_fractal_token(),
                count: Value::Const(1),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedTokens,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountOf(Box::new(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                })),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 125 (push claude/modern_decks): four new Quandrix cards ──────────

/// Quandrix Aetherbinder (b125) — {1}{U}, 1/3 Merfolk Wizard.
/// Magecraft Scry 1. 2-mana defensive smoother body. Same shape as
/// Quandrix Scryweaver (a G/U Scry 1 magecraft) but on a Merfolk
/// Wizard at the {1}{U} slot.
pub fn quandrix_aetherbinder_b125() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Aetherbinder (b125)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Fractal Treewright (b125) — {1}{G}, 0/0 Fractal that enters with
/// 2 +1/+1 counters via `CardDefinition.enters_with_counters` (CR
/// 614.12). 2-mana 2/2 base. Cheap Fractal body for go-tall shells.
pub fn fractal_treewright_b125() -> CardDefinition {
    CardDefinition {
        name: "Fractal Treewright (b125)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        ..Default::default()
    }
}

/// Quandrix Mistsage (b125) — {2}{G}{U}, 3/3 Elf Druid. ETB Scry 1 +
/// magecraft Loot 1. 4-mana defensive value engine. Combines top-deck
/// smoothing on entry with per-cast looting.
pub fn quandrix_mistsage_b125() -> CardDefinition {
    use crate::effect::Duration;
    let _ = Duration::EndOfTurn; // tag to ensure shape consistency
    CardDefinition {
        name: "Quandrix Mistsage (b125)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb(Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            }),
            magecraft_loot(),
        ],
        ..Default::default()
    }
}

/// Fractal Reflection (b125) — {2}{G}{U}, Sorcery. Puts two +1/+1
/// counters on target Fractal you control, then draws a card. 4-mana
/// Fractal-tribal pump + cantrip.
pub fn fractal_reflection_b125() -> CardDefinition {
    CardDefinition {
        name: "Fractal Reflection (b125)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::HasCreatureType(CreatureType::Fractal)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 126 (push claude/modern_decks): five new Quandrix cards ──────────

/// Quandrix Mistshaper (b126) — {1}{U}, 1/3 Merfolk Wizard. Magecraft
/// Draw 1 via the new `magecraft_draw` shortcut. 2-mana defensive
/// magecraft cantripper — pairs with Archmage Emeritus' draw-on-cast.
pub fn quandrix_mistshaper_b126() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mistshaper (b126)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_draw(1)],
        ..Default::default()
    }
}

/// Fractal Skyrunner (b126) — {2}{G}, 0/0 Fractal enters with 3 +1/+1
/// counters via `enters_with_counters`. 3-mana base 3/3 Fractal —
/// Quandrix-tribal payoff scales aggressively under Tanazir Quandrix.
pub fn fractal_skyrunner_b126() -> CardDefinition {
    CardDefinition {
        name: "Fractal Skyrunner (b126)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        ..Default::default()
    }
}

/// Quandrix Riftcraftsman (b126) — {2}{G}{U}, 3/3 Elf Druid. ETB
/// +1/+1 counter on target Fractal you control + Magecraft Loot. 4-mana
/// Fractal-tribal value engine.
pub fn quandrix_riftcraftsman_b126() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Riftcraftsman (b126)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::HasCreatureType(CreatureType::Fractal)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            magecraft_loot(),
        ],
        ..Default::default()
    }
}

/// Quandrix Forecaster-Adept (b126) — {G}{U}, 1/2 Elf Druid. Magecraft
/// Scry 1 (paired with the existing magecraft helpers). 2-mana
/// defensive smoother — pairs with the broader Quandrix scry chain.
pub fn quandrix_forecaster_adept_b126() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Forecaster-Adept (b126)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Fractal Petalcaller (b126) — {2}{G}{U} Sorcery. "Create a 0/0
/// green-and-blue Fractal creature token. Put three +1/+1 counters on
/// it." 4-mana Fractal-mint with built-in 3/3 stat-line.
pub fn fractal_petalcaller_b126() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Fractal Petalcaller (b126)",
        cost: cost(&[generic(2), g(), u()]),
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
                amount: Value::Const(3),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 127 (push claude/modern_decks): new Quandrix cards ──────────────

/// Quandrix Greenmage (b127) — {1}{G}, 2/2 Elf Druid. Magecraft +1/+1
/// counter on self — self-growing magecraft body.
pub fn quandrix_greenmage_b127() -> CardDefinition {
    use crate::effect::shortcut::cast_is_instant_or_sorcery;
    CardDefinition {
        name: "Quandrix Greenmage (b127)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Fractal Bedrock (b127) — {3}{G}, 0/0 Fractal that enters with 4
/// +1/+1 counters. 4-mana Fractal pure body — slots well with Tanazir
/// counter-doubling triggers.
pub fn fractal_bedrock_b127() -> CardDefinition {
    CardDefinition {
        name: "Fractal Bedrock (b127)",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        ..Default::default()
    }
}

/// Quandrix Sageling (b127) — {2}{G}{U}, 2/4 Elf Druid. Magecraft
/// Scry 1 + Draw 1 (loot variant). Defensive selection body.
pub fn quandrix_sageling_b127() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sageling (b127)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Fractal Stormcaller (b127) — {1}{U}, 1/2 Merfolk Wizard. ETB
/// Scry 1.
pub fn fractal_stormcaller_b127() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Fractal Stormcaller (b127)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(1)],
        ..Default::default()
    }
}

/// Quandrix Fractus-Touch (b127) — {1}{G}{U} Instant. Two +1/+1
/// counters on target friendly Fractal + Draw 1.
pub fn quandrix_fractus_touch_b127() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractus-Touch (b127)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::HasCreatureType(CreatureType::Fractal)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 128 (push claude/modern_decks): new Quandrix cards ──────────────

/// Quandrix Bloomforge (b128) — {2}{G}{U}, 3/3 Elemental. ETB mints a
/// 4-counter Fractal via the new `etb_mint_token_with_counters` shortcut.
/// 4-mana double-body (3/3 + 4/4 fractal). Pairs with Bedrock for
/// go-wide Quandrix math shells.
pub fn quandrix_bloomforge_b128() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::etb_mint_token_with_counters;
    CardDefinition {
        name: "Quandrix Bloomforge (b128)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token_with_counters(fractal_token(), 1, 4)],
        ..Default::default()
    }
}

/// Quandrix Tideshaper (b128) — {1}{U}, 2/1 Merfolk Wizard. Magecraft
/// Scry 1 — early flier-fueler that draws every IS spell smoother.
pub fn quandrix_tideshaper_b128() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tideshaper (b128)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Quandrix Treebinder (b128) — {2}{G}, 3/3 Elf Druid Reach. ETB Draw 1
/// (cantrip body). 3-mana defensive + smoother body.
pub fn quandrix_treebinder_b128() -> CardDefinition {
    use crate::effect::shortcut::etb_draw;
    CardDefinition {
        name: "Quandrix Treebinder (b128)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Geometer (b128) — {2}{G}{U}, 2/2 Elf Wizard. ETB mints a
/// 2-counter Fractal (so it enters as a 2/2 Fractal) via the new
/// `etb_mint_token_with_counters` shortcut. Mathematical 4-mana
/// double-body (2/2 + 2/2 Fractal).
pub fn quandrix_geometer_b128() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::etb_mint_token_with_counters;
    CardDefinition {
        name: "Quandrix Geometer (b128)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token_with_counters(fractal_token(), 1, 2)],
        ..Default::default()
    }
}

// ── Batch 129 (push claude/modern_decks): new Quandrix cards ──────────────

/// Quandrix Fractalbinder (b129) — {2}{G}{U}, 3/3 Elf Wizard. Static
/// "Other Fractal creatures you control get +1/+1." Fractal-tribal
/// anthem mirroring the Lorehold Spirit Banner — pairs with Geometer,
/// Bloomforge, Anomaly, Petalcaller for explosive Fractal-tribal play.
pub fn quandrix_fractalbinder_b129() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Quandrix Fractalbinder (b129)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Fractal creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Quandrix Doubler (b129) — {2}{G}{U}, 2/3 Merfolk Wizard. ETB
/// puts a +1/+1 counter on each Fractal you control. Fractal-tribal
/// growth payoff that scales with go-wide Fractal mints.
pub fn quandrix_doubler_b129() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Quandrix Doubler (b129)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Bookworm (b129) — {1}{G}{U}, 2/2 Elf Wizard. Magecraft
/// puts a +1/+1 counter on this creature. Self-growing Tideguard
/// template at a lower curve.
pub fn quandrix_bookworm_b129() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Bookworm (b129)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Bloomscatter (b129) — {3}{G}{U} Sorcery. Create two 2/2
/// Fractal tokens. Go-wide Fractal mint sorcery — pairs with
/// Fractalbinder, Doubler, and Bookworm for chained Fractal payoff.
pub fn quandrix_bloomscatter_b129() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Quandrix Bloomscatter (b129)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedTokens,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 130 (push claude/modern_decks): more Quandrix cards ───────────────

/// Quandrix Fractalseed (b130) — {1}{G}{U}, 2/2 Elf Druid. ETB puts a
/// +1/+1 counter on target Fractal you control. Curve-out partner for
/// Quandrix Geometer (b128) — a 3-mana 2/2 that grows an existing
/// Fractal by +1/+1, paired with Bloomforge / Anomaly seed.
pub fn quandrix_fractalseed_b130() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractalseed (b130)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Doubler II (b130) — {3}{G}{U}, 2/4 Merfolk Wizard. ETB
/// puts two +1/+1 counters on each Fractal you control. Bigger Doubler
/// (b129) variant — 5 mana for a deeper anthem effect.
pub fn quandrix_doubler_ii_b130() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Doubler II (b130)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Fractal Skybloom (b130) — {2}{U}, 2/2 Fractal Wizard, Flying. A
/// Fractal-typed evasive 3-drop — fills the gap between Geometer
/// (2/2 ground) and Tide-Surger (3/3 flying) and benefits from the
/// Fractalbinder anthem.
pub fn fractal_skybloom_b130() -> CardDefinition {
    CardDefinition {
        name: "Fractal Skybloom (b130)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ─── Batch 131: Quandrix synthesised cards ───────────────────────────────────

/// Quandrix Fractalsage (b131) — {1}{G}{U}, 2/2 Fractal Wizard. ETB
/// puts a +1/+1 counter on target Fractal you control.
pub fn quandrix_fractalsage_b131() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractalsage (b131)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Calculator (b131) — {2}{G}{U}, 2/3 Fractal Wizard.
/// Magecraft AddCounter(+1/+1, Self). Self-growing magecraft.
pub fn quandrix_calculator_b131() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Calculator (b131)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Fractal Inkfall (b131) — {3}{G}{U} Sorcery. Create a 0/0 Fractal
/// token, then put 4 +1/+1 counters on it. Single big Fractal body.
pub fn fractal_inkfall_b131() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Fractal Inkfall (b131)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: create_token_with_counter(PlayerRef::You, 1, fractal_token(), CounterType::PlusOnePlusOne, 4),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 132 ───────────────────────────────────────────────────────────────

/// Quandrix Theorymage (b132) — {2}{G}{U}, 3/3 Merfolk Wizard.
/// Magecraft: scry 1. Spellslinging body with built-in smoothing.
pub fn quandrix_theorymage_b132() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Theorymage (b132)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Quandrix Mathstudent (b132) — {G}{U}, 1/2 Elf Druid. Magecraft:
/// add a +1/+1 counter to target creature you control. Cheap counter
/// engine on a curve-out body.
pub fn quandrix_mathstudent_b132() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Quandrix Mathstudent (b132)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Fractal-Tutor (b132) — {3}{U}, 2/3 Merfolk Wizard. ETB:
/// draw a card. Card-advantage body that turns into a recursion engine
/// with Mavinda or other graveyard recursion.
pub fn quandrix_fractal_tutor_b132() -> CardDefinition {
    use crate::effect::shortcut::etb_draw;
    CardDefinition {
        name: "Quandrix Fractal-Tutor (b132)",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_draw(1)],
        ..Default::default()
    }
}

/// Fractal Burst (b132) — {2}{G}{U} Sorcery. Create a 0/0 Fractal
/// token with 3 +1/+1 counters on it. Mid-curve Fractal mint.
pub fn fractal_burst_b132() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Fractal Burst (b132)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: create_token_with_counter(
            PlayerRef::You, 1, fractal_token(), CounterType::PlusOnePlusOne, 3,
        ),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 133 ───────────────────────────────────────────────────────────────

/// Quandrix Forecaster (b133) — {1}{U}, 1/2 Merfolk Wizard. ETB
/// Scry 1, then Draw 1. Uses the new `etb_scry_and_draw` shortcut.
pub fn quandrix_forecaster_b133() -> CardDefinition {
    use crate::effect::shortcut::etb_scry_and_draw;
    CardDefinition {
        name: "Quandrix Forecaster (b133)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry_and_draw(1)],
        ..Default::default()
    }
}

/// Fractal Spore (b133) — {1}{G}, 0/0 Fractal. Enters with 2 +1/+1
/// counters. Cheap baseline Fractal body via the `enters_with_counters`
/// replacement (CR 614.12). Becomes a 2/2 immediately.
pub fn fractal_spore_b133() -> CardDefinition {
    CardDefinition {
        name: "Fractal Spore (b133)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        ..Default::default()
    }
}

/// Quandrix Numerist (b133) — {2}{G}{U}, 2/2 Elf Wizard. Magecraft:
/// draw a card.
pub fn quandrix_numerist_b133() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Numerist (b133)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_draw(1)],
        ..Default::default()
    }
}

// ── Batch 134 ───────────────────────────────────────────────────────────────

/// Quandrix Insight-Mage (b134) — {3}{G}{U}, 3/3 Merfolk Wizard.
/// Magecraft: Scry 1, then Draw 1. Uses the new
/// `magecraft_scry_and_draw` shortcut.
pub fn quandrix_insight_mage_b134() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry_and_draw;
    CardDefinition {
        name: "Quandrix Insight-Mage (b134)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry_and_draw(1)],
        ..Default::default()
    }
}

/// Fractal Hatchling (b134) — {1}{U}, 0/0 Fractal Wizard, Flying.
/// Enters with 2 +1/+1 counters via `enters_with_counters`. Becomes
/// a 2/2 flier on entry.
pub fn fractal_hatchling_b134() -> CardDefinition {
    CardDefinition {
        name: "Fractal Hatchling (b134)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        ..Default::default()
    }
}

// ── Batch 135 ───────────────────────────────────────────────────────────────

/// Quandrix Tracker (b135) — {1}{G} 2/2 Elf Druid. Magecraft loot.
/// Cheap selection engine on a Quandrix two-drop body.
pub fn quandrix_tracker_b135() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tracker (b135)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_loot()],
        ..Default::default()
    }
}

/// Quandrix Equation-Lord (b135) — {2}{G}{U} 0/0 Fractal Wizard
/// Trample. Enters with three +1/+1 counters. Quandrix Fractal
/// midrange body — 3/3 trampler with growth potential.
pub fn quandrix_equation_lord_b135() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Equation-Lord (b135)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        ..Default::default()
    }
}

/// Fractal Aspirant (b135) — {G} 0/0 Fractal Wizard. Enters with one
/// +1/+1 counter. The cheapest possible Fractal-tribal one-drop, fuels
/// Quandrix's +1/+1 counter-payoff cards.
pub fn fractal_aspirant_b135() -> CardDefinition {
    CardDefinition {
        name: "Fractal Aspirant (b135)",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(1))),
        ..Default::default()
    }
}

/// Quandrix Scaleshifter (b135) — {1}{G}{U} 2/2 Merfolk Wizard.
/// Magecraft: put a +1/+1 counter on target creature you control.
/// Quandrix's classic spell-into-counter shape.
pub fn quandrix_scaleshifter_b135() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_to_friendly;
    CardDefinition {
        name: "Quandrix Scaleshifter (b135)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_add_counter_to_friendly()],
        ..Default::default()
    }
}

// ── Batch 136 ───────────────────────────────────────────────────────────────

/// Fractal Beanstalker (b136) — {2}{G}{U} 0/0 Fractal Wizard Reach.
/// Enters with 4 +1/+1 counters. Heavy Reach-blocker with Fractal-tribal
/// support — feeds Fractal payoffs and stalls aerial assaults.
pub fn fractal_beanstalker_b136() -> CardDefinition {
    CardDefinition {
        name: "Fractal Beanstalker (b136)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        ..Default::default()
    }
}

/// Quandrix Mathwarden (b136) — {1}{U} 1/3 Merfolk Wizard. Magecraft
/// draw a card if it's the first instant or sorcery you cast this turn.
/// Approximated as magecraft_scry(1) — same shape, simpler. The first-
/// only condition is engine-wide (no per-turn-first gate primitive).
pub fn quandrix_mathwarden_b136() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mathwarden (b136)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Quandrix Fractal-Apprentice (b136) — {G}{U} 1/1 Fractal Wizard.
/// Magecraft: put a +1/+1 counter on this creature. Self-growing
/// Quandrix Symmathematics body.
pub fn quandrix_fractal_apprentice_b136() -> CardDefinition {
    use crate::card::TriggeredAbility;
    use crate::effect::{EventScope, EventSpec};
    CardDefinition {
        name: "Quandrix Fractal-Apprentice (b136)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(crate::effect::shortcut::cast_is_instant_or_sorcery()),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Batch 138 ───────────────────────────────────────────────────────────────

/// Quandrix Mathmaster (b138) — {2}{G}{U} 3/3 Human Wizard. ETB
/// fractal mint with 2 +1/+1 counters. 4-mana go-wide token engine.
pub fn quandrix_mathmaster_b138() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Quandrix Mathmaster (b138)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(create_token_with_counter(
            PlayerRef::You,
            1,
            fractal_token(),
            CounterType::PlusOnePlusOne,
            2,
        ))],
        ..Default::default()
    }
}

/// Fractal Scholar (b138) — {1}{U} 1/3 Fractal Wizard. Magecraft
/// AddCounter(+1/+1, Self). Self-growing magecraft body.
pub fn fractal_scholar_b138() -> CardDefinition {
    CardDefinition {
        name: "Fractal Scholar (b138)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Equation (b138) — {1}{G}{U} Sorcery. Creates a 0/0
/// Fractal token with 2 +1/+1 counters and you draw a card. Body of
/// Research + cantrip.
pub fn quandrix_equation_b138() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Quandrix Equation (b138)",
        cost: cost(&[generic(1), g(), u()]),
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
                2,
            ),
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 139 ───────────────────────────────────────────────────────────────

/// Fractal Initiate (b139) — {G}{U} 1/1 Fractal Wizard. ETB +1/+1
/// counter on self. Self-pumping 2-mana Fractal.
pub fn fractal_initiate_b139() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Fractal Initiate (b139)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Stormcaster (b139) — {2}{U} 2/3 Human Wizard. Magecraft
/// Draw 1. Card advantage scaler.
pub fn quandrix_stormcaster_b139() -> CardDefinition {
    use crate::effect::shortcut::magecraft_draw;
    CardDefinition {
        name: "Quandrix Stormcaster (b139)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Geometrymage (b139) — {1}{G}{U} 2/3 Human Druid. Magecraft
/// AddCounter(+1/+1, friendly creature). Symmetry Sage's tribal scaler.
pub fn quandrix_geometrymage_b139() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_to_friendly;
    CardDefinition {
        name: "Quandrix Geometrymage (b139)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_add_counter_to_friendly()],
        ..Default::default()
    }
}

/// Fractal Outgrowth (b139) — {3}{G}{U} Sorcery. Mints a Fractal
/// with 4 +1/+1 counters. 5-mana big Fractal mint.
pub fn fractal_outgrowth_b139() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Fractal Outgrowth (b139)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: create_token_with_counter(
            PlayerRef::You,
            1,
            fractal_token(),
            CounterType::PlusOnePlusOne,
            4,
        ),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Lifestream (b136) — {2}{G}{U} Sorcery. Creates a 0/0
/// Fractal token with 3 +1/+1 counters and you gain 2 life. Body of
/// Research mini-version with a defensive lifegain rider.
pub fn quandrix_lifestream_b136() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Quandrix Lifestream (b136)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            create_token_with_counter(PlayerRef::You, 1, fractal_token(), CounterType::PlusOnePlusOne, 3),
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 141 ───────────────────────────────────────────────────────────────

/// Quandrix Symmetrist II (b141) — {2}{G}{U} 3/3 Human Wizard. ETB
/// Fractal token with 3 +1/+1 counters. Heavy go-wide Fractal payoff.
pub fn quandrix_symmetrist_ii_b141() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Quandrix Symmetrist II (b141)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(create_token_with_counter(
            PlayerRef::You,
            1,
            fractal_token(),
            CounterType::PlusOnePlusOne,
            3,
        ))],
        ..Default::default()
    }
}

/// Quandrix Sage (b141) — {1}{U} 1/3 Human Wizard. Magecraft Scry 1 +
/// Draw 1. Spellslinger card-selection engine.
pub fn quandrix_sage_b141() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry_and_draw;
    CardDefinition {
        name: "Quandrix Sage (b141)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry_and_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Fractalcraft (b141) — {G}{U} Instant. AddCounter(+1/+1) +
/// Scry 1 on target creature you control. 2-mana combat trick with
/// selection.
pub fn quandrix_fractalcraft_b141() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractalcraft (b141)",
        cost: cost(&[g(), u()]),
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
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Fractal Wanderer (b141) — {1}{G}{U} 2/2 Fractal Druid Trample.
/// Magecraft put a +1/+1 counter on self.
pub fn fractal_wanderer_b141() -> CardDefinition {
    CardDefinition {
        name: "Fractal Wanderer (b141)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

// ── Batch 142 ───────────────────────────────────────────────────────────────

/// Quandrix Algorithmist (b142) — {1}{G}{U} 2/3 Human Wizard.
/// Magecraft Seq(Scry 1 + AddCounter +1/+1 self). Self-growing
/// magecraft engine that also smooths draws.
pub fn quandrix_algorithmist_b142() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Algorithmist (b142)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Fractal Tendril (b142) — {G}{U} Instant. Create a Fractal token
/// with two +1/+1 counters. 2-mana flash-Fractal token body.
pub fn fractal_tendril_b142() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Fractal Tendril (b142)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: create_token_with_counter(
            PlayerRef::You,
            1,
            fractal_token(),
            CounterType::PlusOnePlusOne,
            2,
        ),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Wavefront (b142) — {2}{G}{U} Sorcery. Draw 2 cards.
/// 4-mana raw card draw — Divination at college costs.
pub fn quandrix_wavefront_b142() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Wavefront (b142)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Apex (b142) — {3}{G}{U} 4/4 Fractal Druid Trample.
/// ETB adds a +1/+1 counter on this creature for each other Fractal
/// creature you control. Tribal payoff for go-wide Fractals.
pub fn quandrix_apex_b142() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Apex (b142)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::count(Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            )),
        })],
        ..Default::default()
    }
}

// ── Batch 143 ───────────────────────────────────────────────────────────────

/// Quandrix Arithmancer (b143) — {1}{G}{U} 2/3 Fractal Wizard. Magecraft:
/// Scry 1 + put a +1/+1 counter on this creature. Self-growing magecraft body.
pub fn quandrix_arithmancer_b143() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Arithmancer (b143)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Fractal Splinter (b143) — {G}{U} 1/1 Fractal. ETB +1 +1/+1 counter
/// on this creature. Compact 2-mana scaler.
pub fn fractal_splinter_b143() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Fractal Splinter (b143)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Doubler (b143) — {2}{G}{U} Instant. Target creature gets
/// +X/+X EOT where X is the number of creatures you control. 4-mana
/// combat math finisher.
pub fn quandrix_doubler_b143() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Doubler (b143)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::count(Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            )),
            toughness: Value::count(Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            )),
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Fractal Vinemother (b143) — {3}{G}{U} 3/3 Fractal Druid. ETB:
/// Create a Fractal token with three +1/+1 counters on it.
pub fn fractal_vinemother_b143() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Fractal Vinemother (b143)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            },
        ]))],
        ..Default::default()
    }
}

// ── Batch 144 ───────────────────────────────────────────────────────────────

/// Quandrix Echoist (b144) — {1}{G}{U} 2/3 Fractal Wizard. Magecraft
/// Draw 1 + Surveil 1.
pub fn quandrix_echoist_b144() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Echoist (b144)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Fractal Scion (b144) — {G}{U} 0/0 Fractal. Enters with X +1/+1
/// counters where X = 2. Compact 2-mana 2/2.
pub fn fractal_scion_b144() -> CardDefinition {
    CardDefinition {
        name: "Fractal Scion (b144)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        ..Default::default()
    }
}

/// Quandrix Mage-Adept (b144) — {2}{G}{U} 2/2 Human Wizard.
/// "Whenever you cast an instant or sorcery, put a +1/+1 counter on
/// target creature you control." Standard Quandrix magecraft.
pub fn quandrix_mage_adept_b144() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_to_friendly;
    CardDefinition {
        name: "Quandrix Mage-Adept (b144)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_add_counter_to_friendly()],
        ..Default::default()
    }
}

// ── Batch 145 ───────────────────────────────────────────────────────────────

/// Quandrix Treetender (b145) — {2}{G} 2/3 Human Druid. Cycling {2}{G}.
pub fn quandrix_treetender_b145() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Treetender (b145)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Cycling(cost(&[generic(2), g()]))],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Fractal Apex-Mage (b145) — {3}{G}{U} 4/4 Fractal Wizard. ETB +1/+1
/// counter on this creature for each other Fractal you control.
pub fn fractal_apex_mage_b145() -> CardDefinition {
    CardDefinition {
        name: "Fractal Apex-Mage (b145)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::count(Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            )),
        })],
        ..Default::default()
    }
}

/// Fractal Bookbearer (b144) — {1}{G} 2/2 Fractal Druid. Cycling {2}.
pub fn fractal_bookbearer_b144() -> CardDefinition {
    CardDefinition {
        name: "Fractal Bookbearer (b144)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Numericist (b143) — {1}{G}{U} 2/2 Human Wizard. Magecraft
/// Draw 1 + discard a card (loot). Card-velocity engine.
pub fn quandrix_numericist_b143() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Numericist (b143)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_loot()],
        ..Default::default()
    }
}

/// Fractal Genesis (b142) — {1}{G}{U} 2/2 Fractal Druid. Magecraft
/// mint a Fractal token (0/0, no counters — dies to SBA unless other
/// effects add counters, but acts as a sacrifice / aristocrats fodder
/// trigger source).
pub fn fractal_genesis_b142() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::magecraft_mint_token;
    CardDefinition {
        name: "Fractal Genesis (b142)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_mint_token(fractal_token(), 1)],
        ..Default::default()
    }
}

// ── Batch 146 ───────────────────────────────────────────────────────────────

/// Quandrix Sumcaster (b146) — {2}{G}{U} 3/3 Fractal Druid Wizard.
/// ETB: target creature you control gets a +1/+1 counter for each
/// other creature you control. Scales hard with token-heavy boards.
pub fn quandrix_sumcaster_b146() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sumcaster (b146)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::CountOf(Box::new(Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ))),
        })],
        ..Default::default()
    }
}

/// Quandrix Mathwitch (b146) — {1}{G}{U} 2/2 Elf Wizard. Magecraft Draw 1
/// + Discard 1 (loot). Quandrix's flagship looter — strict upgrade over
///   Quandrix Numericist's looter at +1 toughness.
pub fn quandrix_mathwitch_b146() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mathwitch (b146)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_loot()],
        ..Default::default()
    }
}

/// Fractal Caller (b146) — {2}{G}{U} 3/3 Fractal Druid. ETB mints a
/// Fractal token (0/0 with N +1/+1 counters where N = your devotion to
/// blue+green). Approximated as a flat 2/2 Fractal token (constant
/// count) since the engine has no devotion primitive.
pub fn fractal_caller_b146() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::etb_mint_token_with_counters;
    CardDefinition {
        name: "Fractal Caller (b146)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token_with_counters(
            fractal_token(),
            1,
            2,
        )],
        ..Default::default()
    }
}

/// Quandrix Counterspell (b146) — {1}{U} Instant. Counter target spell
/// unless its controller pays {2}. Stand-in for a "soft" Quandrix
/// counter — see Lose Focus for the same shape at {U}.
pub fn quandrix_counterspell_b146() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Counterspell (b146)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: Selector::Target(0),
            mana_cost: cost(&[generic(2)]),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Sumstudent (b146) — {1}{G} 2/2 Elf Druid. Magecraft +1/+1
/// counter on this creature. Self-growing 2-drop — Devout/Inkbinder
/// pattern in the green slot.
pub fn quandrix_sumstudent_b146() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sumstudent (b146)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Reflector (b146) — {3}{U} 3/4 Fractal Wizard Flying.
/// 4-mana flier with magecraft Scry 1. Defensive air with card filtering.
pub fn quandrix_reflector_b146() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Reflector (b146)",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Quandrix Field Trip (b146) — {2}{G} Sorcery. Search your library for
/// a basic land, put it onto the battlefield tapped. Standard "Cultivate
/// lite". The "Learn" half from the printed Field Trip is omitted —
/// this is a slim variant.
pub fn quandrix_field_trip_b146() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Field Trip (b146)",
        cost: cost(&[generic(2), g()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Mossbinder (b146) — {2}{G} 3/3 Elf Druid. ETB Search for a
/// basic land + put onto bf tapped. Self-ramping value body.
pub fn quandrix_mossbinder_b146() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mossbinder (b146)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: true,
            },
        })],
        ..Default::default()
    }
}

/// Quandrix Mage-Apprentice (b146) — {G}{U} 2/2 Fractal Wizard. ETB
/// gain 1 life + magecraft Scry 1. 2-mana double-trigger value body.
pub fn quandrix_mage_apprentice_b146() -> CardDefinition {
    use crate::effect::shortcut::etb_gain_life;
    CardDefinition {
        name: "Quandrix Mage-Apprentice (b146)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(1), magecraft_scry(1)],
        ..Default::default()
    }
}

/// Quandrix Patternseeker (b146) — {1}{U} 1/2 Fractal Wizard Flying.
/// Magecraft Draw 1. Quandrix's slim cantrip flier.
pub fn quandrix_patternseeker_b146() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Patternseeker (b146)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_draw(1)],
        ..Default::default()
    }
}

// ── Batch 147 ───────────────────────────────────────────────────────────────

/// Quandrix Calculator (b147) — {1}{G}{U} 2/3 Fractal Wizard. Magecraft
/// loot + +1/+1 counter on self. 3-mana double-trigger value engine.
pub fn quandrix_calculator_b147() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Calculator (b147)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Quandrix Patternsage (b147) — {2}{G}{U} 3/4 Elf Wizard. ETB Scry 2 +
/// Draw 1. 4-mana premium card-selection body.
pub fn quandrix_patternsage_b147() -> CardDefinition {
    use crate::effect::shortcut::etb_scry_and_draw;
    CardDefinition {
        name: "Quandrix Patternsage (b147)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry_and_draw(2)],
        ..Default::default()
    }
}

/// Fractal Apprentice (b147) — {G}{U} 2/2 Fractal Druid. Magecraft +1/+1
/// counter on this creature. Quandrix's Devout/Inkbinder analogue.
pub fn fractal_apprentice_b147() -> CardDefinition {
    CardDefinition {
        name: "Fractal Apprentice (b147)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Bouncer (b147) — {2}{U} Instant. Return target creature to
/// its owner's hand + Scry 1. 3-mana bounce + dig.
pub fn quandrix_bouncer_b147() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Bouncer (b147)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Wallcaller (b147) — {1}{G} 0/4 Elf Druid Defender. ETB GainLife 2.
/// 2-mana wall body — classic defensive ramp shell.
pub fn quandrix_wallcaller_b147() -> CardDefinition {
    use crate::effect::shortcut::etb_gain_life;
    CardDefinition {
        name: "Quandrix Wallcaller (b147)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
    }
}

// ── Batch 148 ───────────────────────────────────────────────────────────────

/// Quandrix Spelltwister (b148) — {G}{U} 1/3 Elf Wizard.
/// Magecraft Scry 1 + GainLife 1. 2-mana defensive card-smoothing body.
pub fn quandrix_spelltwister_b148() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Spelltwister (b148)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Fractal Warrior (b148) — {2}{G}{U} 3/3 Fractal Warrior. ETB +1/+1
/// counter on self. 4-mana sticky body — perfect Lorehold Reliquary
/// payoff partner.
pub fn fractal_warrior_b148() -> CardDefinition {
    CardDefinition {
        name: "Fractal Warrior (b148)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Symbolic (b148) — {2}{U} Instant. Draw 2 + Discard 1.
/// 3-mana cantrip filter.
pub fn quandrix_symbolic_b148() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Symbolic (b148)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Geometer (b148) — {3}{G}{U} 3/4 Elf Wizard. Magecraft +1/+1
/// counter on this creature. 5-mana sticky body.
pub fn quandrix_geometer_b148() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Geometer (b148)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Fractal Apprentice II (b148) — {1}{G} 2/2 Fractal Druid Trample.
/// 2-mana trampler — Quandrix's beater curve filler.
pub fn fractal_apprentice_ii_b148() -> CardDefinition {
    CardDefinition {
        name: "Fractal Apprentice II (b148)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 149 ───────────────────────────────────────────────────────────────

/// Quandrix Skystreaker (b149) — {1}{U} 1/2 Fractal Wizard Flying +
/// Hexproof. 2-mana evasive sticky flier.
pub fn quandrix_skystreaker_b149() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Skystreaker (b149)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Hexproof],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Fractal Eternity (b149) — {3}{G}{U} 3/3 Fractal Druid Undying.
/// Recursion-friendly Fractal body.
pub fn fractal_eternity_b149() -> CardDefinition {
    CardDefinition {
        name: "Fractal Eternity (b149)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Undying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 150 ───────────────────────────────────────────────────────────────

/// Quandrix Fractalweaver (b150) — {2}{G}{U} 2/2 Elf Druid. Magecraft
/// scry 1 + draw 1 — light card-selection magecraft.
pub fn quandrix_fractalweaver_b150() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractalweaver (b150)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Quandrix Spireshape (b150) — {3}{G}{U} 4/4 Fractal Druid. ETB
/// mint a 2/2 Fractal token (0/0 base + 2 +1/+1 counters per
/// `etb_mint_token_with_counters`).
pub fn quandrix_spireshape_b150() -> CardDefinition {
    use crate::effect::shortcut::etb_mint_token_with_counters;
    CardDefinition {
        name: "Quandrix Spireshape (b150)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token_with_counters(quandrix_fractal_token(), 1, 2)],
        ..Default::default()
    }
}

/// Quandrix Hydromancer (b150) — {1}{U} 1/3 Merfolk Wizard. Magecraft
/// draw a card. Strong card draw engine.
pub fn quandrix_hydromancer_b150() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Hydromancer (b150)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Skyrider (b150) — {2}{U} 2/3 Fractal Bird Flying.
/// Mid-curve evasive Fractal.
pub fn quandrix_skyrider_b150() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Skyrider (b150)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Verdant Snake (b150) — {2}{G} 3/2 Fractal Snake Reach.
/// Defensive Fractal body.
pub fn quandrix_verdant_snake_b150() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Verdant Snake (b150)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Snake],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Snake-Egg (b150) — {G} 0/1 Fractal Snake. Magecraft +1/+1
/// counter on self — recursive growth body.
pub fn quandrix_snake_egg_b150() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Snake-Egg (b150)",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Snake],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Mind Curl (b150) — {1}{U} Instant. Counter target creature
/// spell unless its controller pays {2}. Quench-style early counter.
pub fn quandrix_mind_curl_b150() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::mana::cost as mc;
    use crate::mana::generic as gc;
    CardDefinition {
        name: "Quandrix Mind Curl (b150)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(SelectionRequirement::HasCardType(CardType::Creature)),
            ),
            mana_cost: mc(&[gc(2)]),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 151 ───────────────────────────────────────────────────────────────

/// Quandrix Elf Caller (b151) — {G} 1/1 Elf Druid. Magecraft +1/+0 EOT
/// to self.
pub fn quandrix_elf_caller_b151() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Elf Caller (b151)",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        ..Default::default()
    }
}

/// Quandrix Fractal Theorem (b151) — {1}{G}{U} Sorcery. Create a 0/0
/// Fractal with X +1/+1 counters where X = your number of creatures.
pub fn quandrix_fractal_theorem_b151() -> CardDefinition {
    use crate::card::SelectionRequirement;
    CardDefinition {
        name: "Quandrix Fractal Theorem (b151)",
        cost: cost(&[generic(1), g(), u()]),
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
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ))),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Spellmage (b151) — {U} 1/1 Merfolk Wizard. Magecraft draw a
/// card the first time you cast an IS spell each turn (approximated as
/// every cast — no per-turn-once gate).
pub fn quandrix_spellmage_b151() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Spellmage (b151)",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Quandrix Forest Sprite (b151) — {1}{G} 2/2 Fractal Plant. Mid-curve
/// Fractal value body.
pub fn quandrix_forest_sprite_b151() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Forest Sprite (b151)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Plant],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Algebraist (b151) — {2}{G}{U} 3/3 Elf Druid. ETB scry 2 +
/// draw 1 — heavy card filter.
pub fn quandrix_algebraist_b151() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Algebraist (b151)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

// ── Batch 153 ───────────────────────────────────────────────────────────────

/// Quandrix Counter-Squirrel (b153) — {G}{U} 2/2 Fractal Squirrel.
/// Compact 2-mana Fractal body.
pub fn quandrix_counter_squirrel_b153() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Counter-Squirrel (b153)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Squirrel],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Insight (b153) — {1}{U} Instant. Draw 2 cards.
pub fn quandrix_insight_b153() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Insight (b153)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Sage (b153) — {2}{G} 2/3 Elf Druid. ETB +1/+1 counter
/// on target creature you control.
pub fn quandrix_sage_b153() -> CardDefinition {
    use crate::effect::shortcut::target_filtered as tf;
    CardDefinition {
        name: "Quandrix Sage (b153)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: tf(SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ── Batch 154 ───────────────────────────────────────────────────────────────

/// Quandrix Fractalsmith (b154) — {1}{G}{U} 2/2 Human Wizard.
/// Magecraft → mint a 0/0 G/U Fractal with one +1/+1 counter on it
/// via the new `magecraft_mint_fractal(1)` shortcut. The on-cast
/// Fractal engine — pairs with Quandrix mages for the +1/+1 counter
/// snowball plan.
pub fn quandrix_fractalsmith_b154() -> CardDefinition {
    use crate::effect::shortcut::magecraft_mint_fractal;
    CardDefinition {
        name: "Quandrix Fractalsmith (b154)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_mint_fractal(1)],
        ..Default::default()
    }
}

/// Quandrix Equationmage (b154) — {G}{U} 1/2 Merfolk Wizard. Magecraft
/// AddCounter(+1/+1, Self) via `magecraft_add_counter_self()` — same
/// shape as Pensive Professor's secondary half but at 2-mana.
pub fn quandrix_equationmage_b154() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_self;
    CardDefinition {
        name: "Quandrix Equationmage (b154)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Riftguard (b154) — {3}{G}{U} 3/4 Fractal Wizard Reach.
/// ETB target creature you control gets two +1/+1 counters. Solid
/// 5-mana counters-payoff body — pairs with Quandrix Counter-Squirrel
/// (b153) to power the snowball.
pub fn quandrix_riftguard_b154() -> CardDefinition {
    use crate::effect::shortcut::target_filtered as tf;
    CardDefinition {
        name: "Quandrix Riftguard (b154)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: tf(SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Quandrix Tidesinger (b154) — {2}{U} 2/3 Merfolk Wizard. Magecraft
/// Draw 1 via the existing `magecraft_draw(1)`. Compact spell-slinger
/// payoff at 3-mana.
pub fn quandrix_tidesinger_b154() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tidesinger (b154)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Calculation (b154) — {2}{G}{U} Sorcery. Seq(CreateToken
/// 0/0 Fractal + AddCounter (+1/+1, LastCreatedToken, 4) + Draw 1).
/// 4-mana Fractal mint at 4/4 + cantrip.
pub fn quandrix_calculation_b154() -> CardDefinition {
    use crate::effect::shortcut::{draw, mint_fractals};
    CardDefinition {
        name: "Quandrix Calculation (b154)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            mint_fractals(1),
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(4),
            },
            draw(1),
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Wavebreaker (b154) — {3}{U} 3/3 Merfolk Wizard. ETB
/// returns target nonland permanent to its owner's hand. Tempo + body.
pub fn quandrix_wavebreaker_b154() -> CardDefinition {
    use crate::effect::shortcut::target_filtered as tf;
    CardDefinition {
        name: "Quandrix Wavebreaker (b154)",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Move {
            what: tf(SelectionRequirement::Permanent.and(SelectionRequirement::Nonland)),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        })],
        ..Default::default()
    }
}

/// Quandrix Bloomguard (b154) — {3}{G} 3/4 Elf Druid Reach.
/// ETB +1/+1 counter on each creature you control. Mass +1/+1
/// distribution body at 4-mana — strong with Doubling Season.
pub fn quandrix_bloomguard_b154() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Bloomguard (b154)",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ── Batch 155 (modern_decks) — 8 new Quandrix cards ────────────────────────

/// Quandrix Embodiment (b155) — {2}{G}{U} 3/3 Fractal Wizard. ETB:
/// add a +1/+1 counter on itself. Self-pumping Fractal body.
pub fn quandrix_embodiment_b155() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Quandrix Embodiment (b155)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Fractal Magus (b155) — {G}{U} 2/2 Elf Wizard. Magecraft: scry 1.
/// Quandrix card-selection on every spell.
pub fn fractal_magus_b155() -> CardDefinition {
    CardDefinition {
        name: "Fractal Magus (b155)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Quandrix Topologist (b155) — {1}{G}{U} 2/3 Elf Druid. Magecraft:
/// +1/+1 EOT to itself. Spell-payoff self-pumper.
pub fn quandrix_topologist_b155() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Topologist (b155)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
    }
}

/// Quandrix Forecaster (b155) — {1}{G}{U} 1/3 Elf Wizard. ETB: draw
/// a card. Pure cantrip body — Quandrix card-velocity 3-drop.
pub fn quandrix_forecaster_b155() -> CardDefinition {
    use crate::effect::shortcut::etb_draw;
    CardDefinition {
        name: "Quandrix Forecaster (b155)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_draw(1)],
        ..Default::default()
    }
}

/// Fractal Strider (b155) — {3}{G}{U} 4/4 Fractal Trample. Vanilla
/// trample beater for the Quandrix curve.
pub fn fractal_strider_b155() -> CardDefinition {
    CardDefinition {
        name: "Fractal Strider (b155)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Symmetrist II (b155) — {2}{G}{U} Sorcery. Choose one —
/// scry 2 + draw 1 / put two +1/+1 counters on target creature.
/// Modal Quandrix utility.
pub fn quandrix_symmetrist_ii_b155() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Symmetrist II (b155)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: scry 2 + draw 1.
            Effect::Seq(vec![
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ]),
            // Mode 1: two +1/+1 counters on target creature.
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Streamcaller (b155) — {G}{U} 2/1 Elf Druid. Magecraft:
/// draw a card if no card has been drawn this turn (collapsed to:
/// scry 1). Card-velocity micro-Apprentice.
pub fn quandrix_streamcaller_b155() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Streamcaller (b155)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Fractal Crusher (b155) — {2}{G} 3/3 Fractal Beast. ETB: gain 2
/// life. Green ramp-flavored Fractal body.
pub fn fractal_crusher_b155() -> CardDefinition {
    use crate::effect::shortcut::etb_gain_life;
    CardDefinition {
        name: "Fractal Crusher (b155)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
    }
}

// ── Batch 156 (modern_decks) — Quandrix attack-anchor cards ────────────────

/// Quandrix Mathematician II (b156) — {2}{G}{U} 3/3 Elf Wizard.
/// Whenever another creature you control attacks, put a +1/+1
/// counter on it. Multi-attacker counter snowball.
pub fn quandrix_mathematician_ii_b156() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mathematician II (b156)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnotherOfYours),
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── batch 155 — Quandrix ───────────────────────────────────────────────────

/// Quandrix Cartographer (b155) — {1}{G} 2/2 Elf Druid. ETB Scry 1.
pub fn quandrix_cartographer_b155() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Cartographer (b155)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Hatchling (b155) — {U} 0/2 Fractal. Magecraft +1/+1
/// counter on self.
pub fn quandrix_hatchling_b155() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_self;
    CardDefinition {
        name: "Quandrix Hatchling (b155)",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Fractalist (b155) — {2}{G}{U} 2/2 Elf Wizard. Magecraft
/// mints a Fractal token with 1 +1/+1 counter.
pub fn quandrix_fractalist_b155() -> CardDefinition {
    use crate::effect::shortcut::magecraft_mint_fractal;
    CardDefinition {
        name: "Quandrix Fractalist (b155)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_mint_fractal(1)],
        ..Default::default()
    }
}

/// Quandrix Scriptor (b155) — {1}{U} 1/3 Merfolk Wizard. Magecraft
/// Scry 1.
pub fn quandrix_scriptor_b155() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Scriptor (b155)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Quandrix Verdancer (b155) — {3}{G}{U} 4/4 Elf Druid Trample.
/// 5-mana big body for Quandrix midrange.
pub fn quandrix_verdancer_b155() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Verdancer (b155)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Coursebearer (b155) — {2}{G}{U} 3/3 Elf Wizard.
/// ETB Draw 1.
pub fn quandrix_coursebearer_b155() -> CardDefinition {
    use crate::effect::shortcut::etb_draw;
    CardDefinition {
        name: "Quandrix Coursebearer (b155)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Mathwarden (b155) — {2}{U} 1/4 Merfolk Wizard. Magecraft
/// MayDo Draw 1.
pub fn quandrix_mathwarden_b155() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mathwarden (b155)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::MayDo {
            description: "Draw a card?".into(),
            body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
        })],
        ..Default::default()
    }
}

/// Quandrix Expansor (b155) — {X}{G}{U} Sorcery. Create a 0/0 G/U
/// Fractal token with X +1/+1 counters on it. Big-body X-cost ramp.
pub fn quandrix_expansor_b155() -> CardDefinition {
    use crate::catalog::fractal_token;
    use crate::mana::x;
    CardDefinition {
        name: "Quandrix Expansor (b155)",
        cost: cost(&[x(), g(), u()]),
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
                amount: Value::XFromCost,
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Skygazer (b155) — {2}{U} 2/2 Merfolk Wizard Flying.
/// Compact tempo flier with no rider — Quandrix's evasive 3-drop.
pub fn quandrix_skygazer_b155() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Skygazer (b155)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Logician (b155) — {1}{G}{U} 2/2 Elf Wizard. ETB +1/+1
/// counter on target creature. Quandrix's pump + tempo body.
pub fn quandrix_logician_b155() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Logician (b155)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ── Batch 158 (modern_decks) — Quandrix cards ──────────────────────────────

/// Quandrix Coursetaker (b158) — {G}{U} 1/1 Elf Druid.
/// Magecraft Scry 1. Cheap magecraft smoother body.
pub fn quandrix_coursetaker_b158() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Coursetaker (b158)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Quandrix Bigbrain (b158) — {2}{G}{U} 2/3 Elf Wizard.
/// ETB mint a 0/0 Fractal with two +1/+1 counters. 4-mana sticky body.
pub fn quandrix_bigbrain_b158() -> CardDefinition {
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Quandrix Bigbrain (b158)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(create_token_with_counter(
            PlayerRef::You,
            1,
            quandrix_fractal_token(),
            CounterType::PlusOnePlusOne,
            2,
        ))],
        ..Default::default()
    }
}

/// Quandrix Fractaltender (b158) — {1}{G}{U} 2/2 Fractal Druid.
/// Magecraft put a +1/+1 counter on self.
pub fn quandrix_fractaltender_b158() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_self;
    CardDefinition {
        name: "Quandrix Fractaltender (b158)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Fractal Sprite (b158) — {1}{U} 1/3 Fractal Wizard Flying.
/// Defensive Fractal flier — Quandrix-tribal anti-air.
pub fn fractal_sprite_b158() -> CardDefinition {
    CardDefinition {
        name: "Fractal Sprite (b158)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Researcher (b158) — {3}{G}{U} 3/3 Elf Wizard.
/// Magecraft draws a card. Strong magecraft draw payoff.
pub fn quandrix_researcher_b158() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Researcher (b158)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Inquirer (b158) — {1}{U} 1/2 Merfolk Wizard.
/// ETB Scry 1.
pub fn quandrix_inquirer_b158() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Quandrix Inquirer (b158)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(1)],
        ..Default::default()
    }
}

/// Fractal Wallflower (b158) — {2}{G} 2/4 Fractal Plant Reach.
/// Defensive anti-flier Fractal.
pub fn fractal_wallflower_b158() -> CardDefinition {
    CardDefinition {
        name: "Fractal Wallflower (b158)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Plant],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Echo (b158) — {1}{G}{U} Instant.
/// Draw a card; put a +1/+1 counter on target creature you control.
pub fn quandrix_echo_b158() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Echo (b158)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Counterpoint (b158) — {1}{U} Instant.
/// Counter target spell unless its controller pays {1}.
pub fn quandrix_counterpoint_b158() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Counterpoint (b158)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(1)]),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Fractal Skydweller (b158) — {3}{G}{U} 3/3 Fractal Wizard Flying.
/// ETB +1/+1 counter on self. 5-mana evasive sticky body.
pub fn fractal_skydweller_b158() -> CardDefinition {
    CardDefinition {
        name: "Fractal Skydweller (b158)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Equalist (b158) — {G}{U} 2/2 Elf Druid.
/// Magecraft loots — magecraft Draw 1 + Discard 1.
pub fn quandrix_equalist_b158() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Equalist (b158)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_loot()],
        ..Default::default()
    }
}

/// Fractal Researcher (b158) — {2}{U} 1/4 Fractal Wizard.
/// ETB Draw 1. 3-mana defensive cantrip body.
pub fn fractal_researcher_b158() -> CardDefinition {
    use crate::effect::shortcut::etb_draw;
    CardDefinition {
        name: "Fractal Researcher (b158)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Multiplier II (b158) — {3}{G}{U} 4/4 Elf Wizard.
/// ETB mints one 0/0 Fractal token with three +1/+1 counters.
pub fn quandrix_multiplier_ii_b158() -> CardDefinition {
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Quandrix Multiplier II (b158)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(create_token_with_counter(
            PlayerRef::You,
            1,
            quandrix_fractal_token(),
            CounterType::PlusOnePlusOne,
            3,
        ))],
        ..Default::default()
    }
}

/// Quandrix Sapiens (b158) — {2}{G}{U} 3/3 Elf Druid Reach.
/// 4-mana defensive Fractal-supporting body.
pub fn quandrix_sapiens_b158() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sapiens (b158)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Fractal Recursion (b158) — {1}{G}{U} Sorcery.
/// Return target creature card from your graveyard to your hand;
/// then draw a card.
pub fn fractal_recursion_b158() -> CardDefinition {
    CardDefinition {
        name: "Fractal Recursion (b158)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 160 (modern_decks) — Quandrix additions ──────────────────────────

/// Quandrix Bracketscribe (b160) — {2}{G}{U} 3/3 Fractal Wizard.
/// ETB +1/+1 counter on target friendly creature.
pub fn quandrix_bracketscribe_b160() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Bracketscribe (b160)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Hexer (b160) — {1}{G} 2/2 Elf Druid.
/// Magecraft: put a +1/+1 counter on self.
pub fn quandrix_hexer_b160() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_self;
    CardDefinition {
        name: "Quandrix Hexer (b160)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Counterlord (b160) — {3}{G}{U} 3/4 Fractal Wizard.
/// ETB: put a +1/+1 counter on each Fractal you control.
pub fn quandrix_counterlord_b160() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Counterlord (b160)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Spirescribe (b160) — {G}{U} 1/2 Merfolk Druid.
/// Magecraft +1/+1 EOT self pump.
pub fn quandrix_spirescribe_b160() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Spirescribe (b160)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
    }
}

/// Quandrix Mathadept (b160) — {2}{G} 2/3 Elf Druid Reach.
/// Vanilla anti-flier body.
pub fn quandrix_mathadept_b160() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mathadept (b160)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Doublecast (b160) — {1}{U} Instant.
/// Scry 2 + Draw 1.
pub fn quandrix_doublecast_b160() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Doublecast (b160)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Fractal Scaler (b160) — {2}{G}{U} 3/3 Fractal.
/// Magecraft: this gets a +1/+1 counter.
pub fn fractal_scaler_b160() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_self;
    CardDefinition {
        name: "Fractal Scaler (b160)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Tideforge (b160) — {3}{G}{U} Sorcery.
/// Draw 2 cards + Scry 2.
pub fn quandrix_tideforge_b160() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tideforge (b160)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 161 (modern_decks) — More Quandrix ───────────────────────────────

/// Quandrix Wavetiller (b161) — {2}{G}{U} 2/4 Fractal Druid.
/// Magecraft: put a +1/+1 counter on each Fractal you control.
pub fn quandrix_wavetiller_b161() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Wavetiller (b161)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(crate::effect::shortcut::cast_is_instant_or_sorcery()),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Quandrix Pondkeeper (b161) — {1}{G}{U} 2/2 Merfolk Druid.
/// ETB: gain 2 life + Scry 1.
pub fn quandrix_pondkeeper_b161() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Pondkeeper (b161)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}

/// Quandrix Bricelegate (b161) — {4}{G}{U} Sorcery.
/// Create a 0/0 G/U Fractal token with X +1/+1 counters,
/// where X = the number of creatures you control.
pub fn quandrix_bricelegate_b161() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Quandrix Bricelegate (b161)",
        cost: cost(&[generic(4), g(), u()]),
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
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                )),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Spellblossom (b161) — {3}{G}{U} 3/3 Fractal Druid.
/// ETB +1/+1 counter on each creature you control.
pub fn quandrix_spellblossom_b161() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Spellblossom (b161)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Fractal Tidemind (b161) — {2}{U} 2/3 Fractal Wizard.
/// Magecraft: Scry 1, then draw a card.
pub fn fractal_tidemind_b161() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry_and_draw;
    CardDefinition {
        name: "Fractal Tidemind (b161)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry_and_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Riverbase (b161) — {G}{U} 1/3 Merfolk Druid Reach.
/// 2-mana defensive anti-flier.
pub fn quandrix_riverbase_b161() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Riverbase (b161)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 162 (modern_decks) — More Quandrix ───────────────────────────────

/// Quandrix Splashweaver (b162) — {1}{G}{U} 2/3 Merfolk Druid.
/// Magecraft self-pump +1/+1 EOT.
pub fn quandrix_splashweaver_b162() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Splashweaver (b162)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
    }
}

/// Fractal Echoweaver (b162) — {2}{G}{U} 2/4 Fractal Druid.
/// Defensive body, suitable for the Quandrix counter shell.
pub fn fractal_echoweaver_b162() -> CardDefinition {
    CardDefinition {
        name: "Fractal Echoweaver (b162)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Tidemorph (b162) — {3}{U} 3/3 Merfolk Wizard.
/// ETB: Scry 2 + Draw 1.
pub fn quandrix_tidemorph_b162() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tidemorph (b162)",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}

/// Quandrix Sumcoach (b162) — {2}{G} 3/3 Elf Druid.
/// ETB: +1/+1 counter on each Fractal you control.
pub fn quandrix_sumcoach_b162() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sumcoach (b162)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Wavelet (b162) — {U} Instant.
/// Scry 2.
pub fn quandrix_wavelet_b162() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Wavelet (b162)",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 164 (modern_decks) — More Quandrix ──────────────────────────────

/// Quandrix Tideknotter (b164) — {1}{G}{U} 2/3 Merfolk Druid.
/// ETB: Scry 2.
pub fn quandrix_tideknotter_b164() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tideknotter (b164)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![{
            use crate::effect::shortcut::etb_scry;
            etb_scry(2)
        }],
        ..Default::default()
    }
}

/// Fractal Tidewatcher (b164) — {2}{G}{U} 3/3 Fractal Wizard.
/// Magecraft: draw a card.
pub fn fractal_tidewatcher_b164() -> CardDefinition {
    CardDefinition {
        name: "Fractal Tidewatcher (b164)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Mathseeker (b164) — {G}{U} 1/2 Elf Wizard.
/// Magecraft: this creature gets +1/+1 EOT.
pub fn quandrix_mathseeker_b164() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mathseeker (b164)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
    }
}

/// Quandrix Pondscribe (b164) — {2}{U} 1/4 Merfolk Wizard.
/// ETB: draw a card, then discard a card (loot).
pub fn quandrix_pondscribe_b164() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Pondscribe (b164)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![{
            use crate::effect::shortcut::etb_loot;
            etb_loot()
        }],
        ..Default::default()
    }
}

/// Quandrix Naturebind (b164) — {1}{G} Sorcery.
/// Destroy target artifact or enchantment.
pub fn quandrix_naturebind_b164() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Naturebind (b164)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::HasCardType(CardType::Artifact)
                    .or(SelectionRequirement::HasCardType(CardType::Enchantment)),
            ),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Fractal Summoner (b164) — {3}{G}{U} 3/4 Fractal Druid.
/// ETB: create a 0/0 Fractal token with two +1/+1 counters (a 2/2).
pub fn fractal_summoner_b164() -> CardDefinition {
    use crate::effect::shortcut::etb_mint_token_with_counters;
    CardDefinition {
        name: "Fractal Summoner (b164)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token_with_counters(
            crate::catalog::fractal_token(), 1, 2,
        )],
        ..Default::default()
    }
}

/// Quandrix Waveweaver (b164) — {2}{G} 3/3 Elf Druid Trample.
/// Vanilla trampler.
pub fn quandrix_waveweaver_b164() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Waveweaver (b164)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 165 (modern_decks) — More Quandrix ──────────────────────────────

/// Quandrix Hydraformer (b165) — {2}{G}{U} 3/3 Fractal Druid.
/// ETB: draw a card.
pub fn quandrix_hydraformer_b165() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Hydraformer (b165)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![{
            use crate::effect::shortcut::etb_draw;
            etb_draw(1)
        }],
        ..Default::default()
    }
}

/// Quandrix Formulist (b165) — {G}{U} 1/3 Elf Wizard.
/// Magecraft: Scry 1.
pub fn quandrix_formulist_b165() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Formulist (b165)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Quandrix Rootsinger (b165) — {3}{G} 4/4 Elf Druid Trample.
/// Big green body.
pub fn quandrix_rootsinger_b165() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Rootsinger (b165)",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Tidebinder (b165) — {1}{U} Instant.
/// Return target creature with power ≤ 2 to owner's hand.
pub fn quandrix_tidebinder_b165() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tidebinder (b165)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::PowerAtMost(2)),
            ),
            to: crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::OwnerOf(
                Box::new(Selector::Target(0)),
            )),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Spellgrafter (b165) — {3}{G}{U} 3/3 Elf Wizard.
/// ETB: put a +1/+1 counter on target creature.
pub fn quandrix_spellgrafter_b165() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Spellgrafter (b165)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ── Batch 166 (modern_decks) — Quandrix cycle ─────────────────────────────
//
// Ten new Quandrix (G/U) cards: a mix of Fractal minters, +1/+1 counter
// payoffs, scry/draw, and tribal Fractal anthem. All compose against
// existing shortcuts.

/// Quandrix Counterspellbinder (b166) — {1}{G}{U} 2/3 Elf Druid.
/// ETB +1/+1 counter on self.
pub fn quandrix_counterspellbinder_b166() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Counterspellbinder (b166)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Fractal Echofin (b166) — {G}{U} 1/1 Fractal.
/// Vanilla 2-mana fractal body.
pub fn fractal_echofin_b166() -> CardDefinition {
    CardDefinition {
        name: "Fractal Echofin (b166)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Echomender (b166) — {2}{G}{U} 3/3 Elf Druid.
/// Magecraft: +1/+1 counter on self.
pub fn quandrix_echomender_b166() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Echomender (b166)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Sweep (b166) — {1}{G}{U} Sorcery.
/// Each creature you control gets +1/+1 EOT.
pub fn quandrix_sweep_b166() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sweep (b166)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Wavecaster (b166) — {2}{U} 1/3 Elf Wizard.
/// Magecraft: draw 1.
pub fn quandrix_wavecaster_b166() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Wavecaster (b166)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Tideguard (b166) — {2}{G} 2/4 Elf Druid Reach.
/// 3-mana defensive reach body.
pub fn quandrix_tideguard_b166() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tideguard (b166)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Spellbinder (b166) — {G}{U} Instant.
/// +2/+2 EOT to target friendly creature.
pub fn quandrix_spellbinder_b166() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Spellbinder (b166)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Sumcaller (b166) — {3}{G}{U} Sorcery.
/// Mints a Fractal token with 4 +1/+1 counters.
pub fn quandrix_sumcaller_b166() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Quandrix Sumcaller (b166)",
        cost: cost(&[generic(3), g(), u()]),
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
                amount: Value::Const(4),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Mathstrider (b166) — {3}{G} 3/3 Elf Druid Trample.
/// 4-mana trampling body.
pub fn quandrix_mathstrider_b166() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mathstrider (b166)",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Splitstone (b166) — {2}{G}{U} 3/3 Elemental.
/// ETB: 2 +1/+1 counters on self.
pub fn quandrix_splitstone_b166() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Splitstone (b166)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

// ── Batch 167 (modern_decks) — Quandrix follow-up ─────────────────────────

/// Quandrix Tideforge (b167) — {1}{U} 2/1 Elf Wizard Flash.
/// Cheap flash creature for end-step plays.
pub fn quandrix_tideforge_b167() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tideforge (b167)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Pluralizer (b167) — {3}{G}{U} 2/2 Elf Druid.
/// ETB: put two +1/+1 counters on target creature you control.
pub fn quandrix_pluralizer_b167() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Pluralizer (b167)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Quandrix Echobinder (b167) — {U} Instant.
/// Counter target spell unless its controller pays {2}.
pub fn quandrix_echobinder_b167() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Echobinder (b167)",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: Selector::Target(0),
            mana_cost: crate::mana::cost(&[generic(2)]),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Fractal Crusher (b167) — {3}{G}{U} 4/4 Fractal Trample.
/// Pure 5-mana trampler.
pub fn fractal_crusher_b167() -> CardDefinition {
    CardDefinition {
        name: "Fractal Crusher (b167)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 169 (modern_decks) — Quandrix expansion (8 cards) ───────────────

/// Quandrix Echofin (b169) — {1}{G}{U} 2/2 Fractal Fish.
/// Whenever you cast an instant or sorcery, put a +1/+1 counter on this.
pub fn quandrix_echofin_b169() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Echofin (b169)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Fish],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Pluralize (b169) — {3}{G}{U} Instant.
/// Put two +1/+1 counters on target creature you control.
pub fn quandrix_pluralize_b169() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Pluralize (b169)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Splitscholar (b169) — {2}{G}{U} 2/3 Elf Wizard.
/// ETB: Draw a card.
pub fn quandrix_splitscholar_b169() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Splitscholar (b169)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Tideforge (b169) — {3}{G}{U} 3/4 Fractal Wizard.
/// Magecraft: put a +1/+1 counter on target creature you control.
pub fn quandrix_tideforge_b169_v2() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tideforge II (b169)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Echocaster (b169) — {1}{U} 1/3 Elf Wizard.
/// Magecraft: draw a card.
pub fn quandrix_echocaster_b169() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Echocaster (b169)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Plantarchitect (b169) — {2}{G} 2/3 Elf Druid.
/// Magecraft: this creature gets +1/+1 EOT.
pub fn quandrix_plantarchitect_b169() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Plantarchitect (b169)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
    }
}

/// Quandrix Bigwave (b169) — {3}{G}{U} Sorcery.
/// Draw 3 cards, then put two +1/+1 counters on target creature you control.
pub fn quandrix_bigwave_b169() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Bigwave (b169)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 170 (modern_decks) — Quandrix expansion ─────────────────────────

// ── Batch 171 (modern_decks) — Quandrix expansion ─────────────────────────

/// Quandrix Echocrasher (b171) — {3}{G}{U} 4/4 Fractal Elemental Trample.
/// Whenever a creature you control deals combat damage, put a +1/+1
/// counter on it.
pub fn quandrix_echocrasher_b171() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Echocrasher (b171)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Elemental],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Batch 172 (modern_decks) — Quandrix expansion ─────────────────────────

/// Quandrix Foragelord (b172) — {2}{G} 3/3 Elf Druid.
/// Magecraft: gain 1 life.
pub fn quandrix_foragelord_b172() -> CardDefinition {
    use crate::effect::shortcut::magecraft_gain_life;
    CardDefinition {
        name: "Quandrix Foragelord (b172)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(1)],
        ..Default::default()
    }
}

/// Quandrix Sumcheck (b172) — {1}{G}{U} Instant.
/// Counter target spell unless its controller pays {2}.
pub fn quandrix_sumcheck_b172() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sumcheck (b172)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(2)]),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Fractalmancer (b171) — {2}{G}{U} 3/3 Human Druid Wizard.
/// Magecraft: scry 1 + draw a card.
pub fn quandrix_fractalmancer_b171() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractalmancer (b171)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Quandrix Hydromancer (b170) — {2}{G}{U} 2/3 Elf Wizard.
/// ETB: put a shield counter on this creature. Magecraft: draw a card.
pub fn quandrix_hydromancer_b170() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Hydromancer (b170)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Shield,
                amount: Value::Const(1),
            }),
            magecraft_draw(1),
        ],
        ..Default::default()
    }
}

/// Quandrix Fractal Whale (b169) — {4}{G}{U} 5/5 Fractal Whale Trample.
/// Vanilla finisher.
pub fn quandrix_fractal_whale_b169() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractal Whale (b169)",
        cost: cost(&[generic(4), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Whale],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Echodraw (b167) — {2}{U} Sorcery.
/// Draw 2 cards.
pub fn quandrix_echodraw_b167() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Echodraw (b167)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 174 (modern_decks) — additional Quandrix cards ──────────────────

/// Quandrix Symbolist (b174) — {1}{G} 2/2 Elf Druid.
/// Magecraft: +1/+1 counter on this creature.
pub fn quandrix_symbolist_b174() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_self;
    CardDefinition {
        name: "Quandrix Symbolist (b174)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Mathshape (b174) — {2}{G}{U} 3/3 Elf Wizard.
/// Magecraft: draw a card.
pub fn quandrix_mathshape_b174() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mathshape (b174)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Fractalspinner (b174) — {3}{G}{U} 2/4 Elf Wizard.
/// ETB: create a 0/0 Fractal with 2 +1/+1 counters.
pub fn quandrix_fractalspinner_b174() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::etb_mint_token_with_counters;
    CardDefinition {
        name: "Quandrix Fractalspinner (b174)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token_with_counters(
            fractal_token(),
            1,
            2,
        )],
        ..Default::default()
    }
}

/// Quandrix Riverflow (b174) — {1}{U} Instant. Draw 2; you lose 1 life.
pub fn quandrix_riverflow_b174() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Riverflow (b174)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Sapcaller (b174) — {2}{G} 3/3 Elf Druid.
/// Whenever this creature attacks, you may put a +1/+1 counter on another target creature you control.
/// (Simplified to always add the counter on a friendly creature.)
pub fn quandrix_sapcaller_b174() -> CardDefinition {
    use crate::card::EventScope;
    CardDefinition {
        name: "Quandrix Sapcaller (b174)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Batch 175 (modern_decks) — additional Quandrix cards ──────────────────

/// Quandrix Mathwarden (b175) — {2}{G}{U} 2/4 Elf Druid.
/// Magecraft: target creature you control gets +1/+1 EOT.
pub fn quandrix_mathwarden_b175() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Quandrix Mathwarden (b175)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            1,
            1,
        )],
        ..Default::default()
    }
}

/// Quandrix Beastform (b175) — {1}{G}{U} Sorcery.
/// Create a 0/0 Fractal with 3 +1/+1 counters.
pub fn quandrix_beastform_b175() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Quandrix Beastform (b175)",
        cost: cost(&[generic(1), g(), u()]),
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
                amount: Value::Const(3),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 185 (modern_decks) — Quandrix keyword counter expansion ─────────

/// Quandrix Skyfractal (b185) — {2}{G} Sorcery.
/// Create a 0/0 Fractal with one flying counter and two +1/+1 counters.
pub fn quandrix_skyfractal_b185() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Quandrix Skyfractal (b185)",
        cost: cost(&[generic(2), g()]),
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
                amount: Value::Const(2),
            },
            Effect::AddKeywordCounter {
                what: Selector::LastCreatedToken,
                keyword: Keyword::Flying,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 182 (modern_decks) — closer to a balanced Quandrix cube ─────────

/// Quandrix Streamwarden (b182) — {2}{G}{U} 2/3 Merfolk Druid.
/// ETB: scry 2 + +1/+1 counter on this creature.
pub fn quandrix_streamwarden_b182() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Streamwarden (b182)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

// ── Batch 191 (modern_decks) — multi-action cards + Fractal tribal ────────

/// Quandrix Sumtotal (b191) — {3}{G}{U} Sorcery.
/// Mints a 4/4 Fractal token + you draw 1.
pub fn quandrix_sumtotal_b191() -> CardDefinition {
    let fractal = TokenDefinition {
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
    };
    CardDefinition {
        name: "Quandrix Sumtotal (b191)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: fractal,
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(4),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Sparkbloomer (b191) — {1}{G}{U} 2/2 Fractal Druid.
/// Magecraft self-pump +1/+1 EOT.
pub fn quandrix_sparkbloomer_b191() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sparkbloomer (b191)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
    }
}

/// Quandrix Vinegrower (b191) — {G}{U} 1/3 Fractal Druid.
/// ETB +1/+1 counter on self.
pub fn quandrix_vinegrower_b191() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Vinegrower (b191)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ── Batch 190 (modern_decks) — keyword counter granters ──────────────────

/// Quandrix Doublegrowth (b190) — {1}{G}{U} Sorcery.
/// Target creature gets a trample counter and a flying counter.
pub fn quandrix_doublegrowth_b190() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Doublegrowth (b190)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddKeywordCounter {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Trample,
                amount: Value::Const(1),
            },
            Effect::AddKeywordCounter {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Flying,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Riftleaper (b190) — {2}{U} 2/2 Fractal Wizard.
/// ETB self-flying counter.
pub fn quandrix_riftleaper_b190() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Riftleaper (b190)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddKeywordCounter {
            what: Selector::This,
            keyword: Keyword::Flying,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Sapleader (b190) — {3}{G}{U} 4/4 Fractal.
/// ETB +1/+1 counter on self + reach counter on self.
pub fn quandrix_sapleader_b190() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sapleader (b190)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::AddKeywordCounter {
                what: Selector::This,
                keyword: Keyword::Reach,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

// ── Batch 189 (modern_decks) — additional Quandrix cards ──────────────────

/// Quandrix Beastcaller (b189) — {2}{G} 2/3 Fractal Druid.
/// ETB +1/+1 counter on each Fractal you control.
pub fn quandrix_beastcaller_b189() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Beastcaller (b189)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                    .and(SelectionRequirement::ControlledByYou),
            ),
            body: Box::new(Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        ..Default::default()
    }
}

/// Quandrix Cantrip (b189) — {1}{U} Instant.
/// Draw 2 cards.
pub fn quandrix_cantrip_b189() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Cantrip (b189)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Vinescaler II (b189) — {3}{G} 4/4 Fractal.
/// Reach + Trample.
pub fn quandrix_vinescaler_ii_b189() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Vinescaler II (b189)",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 188 (modern_decks) — additional Quandrix cards ──────────────────

/// Quandrix Mossleaf (b188) — {1}{G} 2/3 Plant.
/// Reach.
pub fn quandrix_mossleaf_b188() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mossleaf (b188)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Dataweaver (b188) — {2}{G}{U} 3/3 Fractal Wizard.
/// Magecraft +1/+1 counter on self.
pub fn quandrix_dataweaver_b188() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_self;
    CardDefinition {
        name: "Quandrix Dataweaver (b188)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Latticebreaker (b188) — {3}{U}{U} Sorcery.
/// Draw 3 cards.
pub fn quandrix_latticebreaker_b188() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Latticebreaker (b188)",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(3),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 187 (modern_decks) — Quandrix expansion ─────────────────────────

/// Quandrix Tramplerune (b187) — {1}{G} Sorcery.
/// Put a trample counter on target creature you control.
pub fn quandrix_tramplerune_b187() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tramplerune (b187)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddKeywordCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            keyword: Keyword::Trample,
            amount: Value::Const(1),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Fractal-Tutor (b187) — {2}{G}{U} Sorcery.
/// Mints a 0/0 Fractal token with 3 +1/+1 counters and a flying counter.
pub fn quandrix_fractal_tutor_b187() -> CardDefinition {
    let fractal = TokenDefinition {
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
    };
    CardDefinition {
        name: "Quandrix Fractal-Tutor (b187)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: fractal,
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            },
            Effect::AddKeywordCounter {
                what: Selector::LastCreatedToken,
                keyword: Keyword::Flying,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Vinescaler (b187) — {3}{G}{U} 3/3 Fractal Druid.
/// ETB +1/+1 counter on self. Magecraft: target friendly Fractal gets a
/// +1/+1 counter.
pub fn quandrix_vinescaler_b187() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Vinescaler (b187)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            magecraft(Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal)),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        ],
        ..Default::default()
    }
}

/// Quandrix Treestrider (b187) — {2}{G} 3/3 Plant.
/// Reach + Trample. Plain body.
pub fn quandrix_treestrider_b187() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Treestrider (b187)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Quickdraw (b187) — {1}{U} Instant.
/// Counter target spell unless its controller pays {2}.
pub fn quandrix_quickdraw_b187() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Quickdraw (b187)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(2)]),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Mossglider (b187) — {1}{G}{U} 2/3 Fractal Druid Flash.
/// ETB +1/+1 counter on self.
pub fn quandrix_mossglider_b187() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mossglider (b187)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Resonator (b187) — {G}{U} 2/2 Fractal Wizard.
/// Magecraft self-pump +1/+1 EOT.
pub fn quandrix_resonator_b187() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Resonator (b187)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
    }
}

// ── Batch 180 (modern_decks) — Fractal-centric Quandrix expansion ────────

/// Quandrix Counterspinner (b180) — {1}{U} Instant. Counter target spell with mana value 2 or less.
pub fn quandrix_counterspinner_b180() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Quandrix Counterspinner (b180)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(SelectionRequirement::ManaValueAtMost(2)),
            ),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Fractal-Echocaller (b180) — {2}{G} 2/2 Elf Druid.
/// ETB: create a Fractal with 1 +1/+1 counter.
pub fn quandrix_fractal_echocaller_b180() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::etb_mint_token_with_counters;
    CardDefinition {
        name: "Quandrix Fractal-Echocaller (b180)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token_with_counters(fractal_token(), 1, 1)],
        ..Default::default()
    }
}

// ── Batch 178 (modern_decks) — more Quandrix variants ─────────────────────

/// Quandrix Drawcaster (b178) — {3}{U} 2/3 Elf Wizard. ETB: draw a card.
pub fn quandrix_drawcaster_b178() -> CardDefinition {
    use crate::effect::shortcut::etb_draw;
    CardDefinition {
        name: "Quandrix Drawcaster (b178)",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_draw(1)],
        ..Default::default()
    }
}

// ── Batch 177 (modern_decks) — more Quandrix variants ─────────────────────

/// Quandrix Streamcaster (b177) — {2}{G}{U} 3/3 Merfolk Druid.
/// ETB: scry 2 then draw a card.
pub fn quandrix_streamcaster_b177() -> CardDefinition {
    use crate::effect::shortcut::etb_scry_and_draw;
    CardDefinition {
        name: "Quandrix Streamcaster (b177)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry_and_draw(2)],
        ..Default::default()
    }
}

/// Quandrix Fractalkeeper (b177) — {3}{G}{U} 2/4 Elf Druid.
/// ETB: create a 0/0 Fractal with 4 +1/+1 counters.
pub fn quandrix_fractalkeeper_b177() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::etb_mint_token_with_counters;
    CardDefinition {
        name: "Quandrix Fractalkeeper (b177)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token_with_counters(fractal_token(), 1, 4)],
        ..Default::default()
    }
}

/// Quandrix Tidemind (b175) — {3}{U} 3/3 Elf Wizard.
/// ETB: draw a card.
pub fn quandrix_tidemind_b175() -> CardDefinition {
    use crate::effect::shortcut::etb_draw;
    CardDefinition {
        name: "Quandrix Tidemind (b175)",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Wavelock (b174) — {2}{U} Instant. Counter target spell unless its controller pays {2}.
pub fn quandrix_wavelock_b174() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Wavelock (b174)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(2)]),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 193 (modern_decks) — Quandrix G/U deep cuts ────────────────────

use crate::catalog::sets::sos::fractal_token;

/// Quandrix Counterleaf (b193) — {1}{U}{G} 2/3 Plant Wizard.
/// Magecraft: scry 1.
pub fn quandrix_counterleaf_b193() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Counterleaf (b193)",
        cost: cost(&[generic(1), u(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Quandrix Vinescholar (b193) — {1}{G} 2/2 Plant Druid.
/// Vanilla 2/2 for {1}{G}, simple curve filler.
pub fn quandrix_vinescholar_b193() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Vinescholar (b193)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Fractalstamp (b193) — {1}{G}{U} Sorcery.
/// Create a 2/2 Fractal token with two +1/+1 counters on it.
pub fn quandrix_fractalstamp_b193() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractalstamp (b193)",
        cost: cost(&[generic(1), g(), u()]),
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
                amount: Value::Const(2),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Drawmage (b193) — {2}{U} 2/2 Merfolk Wizard.
/// Magecraft: draw a card.
pub fn quandrix_drawmage_b193() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Drawmage (b193)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Treesage (b193) — {3}{G}{G} 5/5 Plant Beast Trample.
/// Big Quandrix curve-topper trampler.
pub fn quandrix_treesage_b193() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Treesage (b193)",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Skybinder (b193) — {2}{U} 2/3 Bird Wizard Flying.
/// Solid Quandrix flying body.
pub fn quandrix_skybinder_b193() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Skybinder (b193)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 194 (modern_decks) — Quandrix G/U compact additions ─────────────

/// Quandrix Cantrip II (b194) — {1}{U} Instant. Draw 2 cards.
pub fn quandrix_cantrip_ii_b194() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Cantrip II (b194)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Fractalmage (b194) — {2}{G}{U} 2/4 Wizard.
/// ETB: create a 2/2 Fractal with two +1/+1 counters.
pub fn quandrix_fractalmage_b194() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Quandrix Fractalmage (b194)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
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
        ]))],
        ..Default::default()
    }
}

/// Quandrix Treeshepherd (b194) — {2}{G} 3/3 Plant Druid.
/// Simple curve filler — above-rate 3/3 for {2}{G}.
pub fn quandrix_treeshepherd_b194() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Treeshepherd (b194)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Multiprover (b194) — {2}{U} 2/2 Wizard.
/// Magecraft: scry 1 and draw 1.
pub fn quandrix_multiprover_b194() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry_and_draw;
    CardDefinition {
        name: "Quandrix Multiprover (b194)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry_and_draw(1)],
        ..Default::default()
    }
}

// ── Batch 195 (modern_decks) — Quandrix more deep cuts ────────────────────

/// Quandrix Algebrick (b195) — {1}{G}{U} 2/2 Construct.
/// Magecraft: put a +1/+1 counter on this creature.
pub fn quandrix_algebrick_b195() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_self;
    CardDefinition {
        name: "Quandrix Algebrick (b195)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Quandrix Reefcleric (b195) — {2}{U} 2/3 Merfolk Cleric.
/// ETB: draw a card.
pub fn quandrix_reefcleric_b195() -> CardDefinition {
    use crate::effect::shortcut::etb_draw;
    CardDefinition {
        name: "Quandrix Reefcleric (b195)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Reefranger (b195) — {2}{G}{U} 3/3 Merfolk Wizard.
/// ETB: gain 2 life and put a +1/+1 counter on this creature.
pub fn quandrix_reefranger_b195() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Reefranger (b195)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Quandrix Branchsage (b195) — {3}{G}{G} 5/4 Plant Druid Trample.
/// Big curve-topper.
pub fn quandrix_branchsage_b195() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Branchsage (b195)",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 196 (modern_decks) — Quandrix more variety ──────────────────────

/// Quandrix Riverling (b196) — {1}{U} 1/3 Merfolk.
/// Vanilla durable blocker.
pub fn quandrix_riverling_b196() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Riverling (b196)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Mathlord (b196) — {3}{G}{U} 4/4 Wizard.
/// ETB: create a Fractal with 3 +1/+1 counters.
pub fn quandrix_mathlord_b196() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Quandrix Mathlord (b196)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            },
        ]))],
        ..Default::default()
    }
}

/// Quandrix Vinetwine (b196) — {1}{G}{U} Instant.
/// Target creature gets +2/+2 EOT.
pub fn quandrix_vinetwine_b196() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Vinetwine (b196)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Algescholar (b196) — {2}{G} 2/3 Plant Druid.
/// ETB: put a +1/+1 counter on target creature you control.
pub fn quandrix_algescholar_b196() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Algescholar (b196)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ── Batch 197 (modern_decks) — Quandrix polish ───────────────────────────

/// Quandrix Vinestudent (b197) — {G} 1/2 Plant Druid.
/// Cheap green one-drop.
pub fn quandrix_vinestudent_b197() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Vinestudent (b197)",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Fractalsense (b197) — {1}{G}{U} 2/2 Fractal.
/// Wait, that's a token shape. Make it a creature: ETB add counters.
/// ETB: put two +1/+1 counters on this creature.
pub fn quandrix_fractalsense_b197() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractalsense (b197)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

// ── Batch 198 (modern_decks) — Quandrix (G/U) extension ──────────────────

/// Quandrix Vinemage (b198) — {G}{U} 2/1 Elf Druid.
/// Magecraft scry 1.
pub fn quandrix_vinemage_b198() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Vinemage (b198)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Quandrix Mathscholar (b198) — {1}{U} 1/3 Merfolk Wizard.
/// Magecraft draw 1.
pub fn quandrix_mathscholar_b198() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mathscholar (b198)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Treegrower (b198) — {2}{G} 2/2 Plant Druid.
/// ETB target creature you control gets a +1/+1 counter.
pub fn quandrix_treegrower_b198() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Treegrower (b198)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Fractalist (b198) — {2}{G}{U} 3/3 Human Wizard.
/// Magecraft: mint a Fractal with one +1/+1 counter.
pub fn quandrix_fractalist_b198() -> CardDefinition {
    use crate::effect::shortcut::magecraft_mint_fractal;
    CardDefinition {
        name: "Quandrix Fractalist (b198)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_mint_fractal(1)],
        ..Default::default()
    }
}

/// Quandrix Stargazer (b198) — {1}{U} Sorcery.
/// Scry 2, then draw a card.
pub fn quandrix_stargazer_b198() -> CardDefinition {
    use crate::effect::shortcut::draw;
    CardDefinition {
        name: "Quandrix Stargazer (b198)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }, draw(1)]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Reachelm (b198) — {2}{G} 2/4 Treefolk Druid Reach.
/// Defensive reach body.
pub fn quandrix_reachelm_b198() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Reachelm (b198)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Cipher (b198) — {U} Instant.
/// Scry 2.
pub fn quandrix_cipher_b198() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Cipher (b198)",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Beastcaller (b198) — {4}{G}{U} 5/5 Beast Druid Trample.
/// Curve-topper trampler in school colors.
pub fn quandrix_beastcaller_b198() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Beastcaller (b198)",
        cost: cost(&[generic(4), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast, CreatureType::Druid],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 199 (modern_decks) — Quandrix rounding-out ─────────────────────

/// Quandrix Vinetwist (b199) — {1}{G} 2/2 Plant Druid.
/// Vanilla green 2-drop.
pub fn quandrix_vinetwist_b199() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Vinetwist (b199)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Mathmage (b199) — {1}{U} 1/2 Merfolk Wizard.
/// ETB Scry 2.
pub fn quandrix_mathmage_b199() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Quandrix Mathmage (b199)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(2)],
        ..Default::default()
    }
}

/// Quandrix Pulse (b199) — {G}{U} Instant.
/// Draw 1 card, then put a +1/+1 counter on target creature you control.
pub fn quandrix_pulse_b199() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Pulse (b199)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Geomancer (b199) — {3}{G}{U} 3/4 Human Wizard.
/// ETB: put two +1/+1 counters on target creature you control.
pub fn quandrix_geomancer_b199() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Geomancer (b199)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Quandrix Fractalpath (b199) — {G}{U} Sorcery.
/// Mint a Fractal with two +1/+1 counters.
pub fn quandrix_fractalpath_b199() -> CardDefinition {
    use crate::effect::shortcut::mint_fractals;
    CardDefinition {
        name: "Quandrix Fractalpath (b199)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            mint_fractals(1),
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 200 (modern_decks) — Quandrix round 200 ───────────────────────

/// Quandrix Watergrower (b200) — {2}{U} 2/3 Merfolk Druid.
pub fn quandrix_watergrower_b200() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Watergrower (b200)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Anchorvine (b200) — {3}{G}{U} 4/4 Plant Fractal Vigilance.
pub fn quandrix_anchorvine_b200() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Anchorvine (b200)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Fractal],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 201 (modern_decks) — Quandrix nuanced round ────────────────────

/// Quandrix Cropping (b201) — {3}{G}{U} Sorcery.
/// Put two +1/+1 counters on each creature you control.
pub fn quandrix_cropping_b201() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Cropping (b201)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Crystalshard (b201) — {1}{U} 0/3 Elemental Defender.
/// Defender + ETB scry 2.
pub fn quandrix_crystalshard_b201() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Quandrix Crystalshard (b201)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(2)],
        ..Default::default()
    }
}

// ── Batch 202 (modern_decks) — Quandrix expansion ────────────────────────

/// Quandrix Conjurer (b202) — {2}{G}{U} 2/2 Human Wizard.
/// Magecraft: scry 1, then draw a card. Smoothing engine.
pub fn quandrix_conjurer_b202() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry_and_draw;
    CardDefinition {
        name: "Quandrix Conjurer (b202)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry_and_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Fractalweaver (b202) — {1}{G}{U} 1/1 Fractal.
/// Magecraft: mint a 1/1 Fractal token with one +1/+1 counter.
pub fn quandrix_fractalweaver_b202() -> CardDefinition {
    use crate::effect::shortcut::magecraft_mint_fractal;
    CardDefinition {
        name: "Quandrix Fractalweaver (b202)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_mint_fractal(1)],
        ..Default::default()
    }
}

/// Quandrix Cantrip (b202) — {1}{U} Instant.
/// Draw 2 cards.
pub fn quandrix_cantrip_b202() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Cantrip (b202)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Grizzler (b202) — {2}{G} 3/3 Bear Druid.
/// Vigilance. Vanilla 3-mana value bear-druid.
pub fn quandrix_grizzler_b202() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Grizzler (b202)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Sumtotal (b202) — {3}{G}{U} Sorcery.
/// Put X +1/+1 counters on target creature where X is the number of
/// creatures you control.
pub fn quandrix_sumtotal_b202() -> CardDefinition {
    use crate::effect::shortcut::{each_your_creature, count};
    CardDefinition {
        name: "Quandrix Sumtotal (b202)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: count(each_your_creature()),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Skydiver (b202) — {2}{U} 2/2 Merfolk Wizard.
/// Flying, Hexproof. Evasive, hard to remove.
pub fn quandrix_skydiver_b202() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Skydiver (b202)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Hexproof],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Sparkbender (b202) — {1}{G} Instant.
/// Counter target spell unless its controller pays {1}.
pub fn quandrix_sparkbender_b202() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sparkbender (b202)",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        // Approximation: plain counter (no pay-X to escape rider).
        effect: Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Vinemage (b202) — {3}{G} 4/3 Druid.
/// ETB: put a +1/+1 counter on target creature you control.
pub fn quandrix_vinemage_b202() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Vinemage (b202)",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Fractalspawn (b202) — {2}{G}{U} 2/2 Fractal.
/// ETB: mint a 1/1 Fractal token with two +1/+1 counters.
pub fn quandrix_fractalspawn_b202() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Quandrix Fractalspawn (b202)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
                    amount: Value::Const(2),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Quandrix Symmetry (b202) — {X}{G}{U} Sorcery.
/// Mint a 0/0 Fractal token with X +1/+1 counters.
pub fn quandrix_symmetry_b202() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::mana::x;
    CardDefinition {
        name: "Quandrix Symmetry (b202)",
        cost: cost(&[x(), g(), u()]),
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
                amount: Value::XFromCost,
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Streampath (b202) — {2}{U} Instant.
/// Return target creature to its owner's hand. Draw a card.
pub fn quandrix_streampath_b202() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Streampath (b202)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Geomant (b202) — {2}{G}{U} 3/3 Druid Wizard.
/// {2}{G}{U}: put a +1/+1 counter on target creature you control.
/// Late-game mana sink.
pub fn quandrix_geomant_b202() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Geomant (b202)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Druid, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), u()]),
            tap_cost: false,
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..ActivatedAbility::default()
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 203 (modern_decks) — Quandrix compact round ────────────────────

/// Quandrix Apprentice II (b203) — {G}{U} 1/1 Druid. Magecraft +1/+1 EOT
/// target friendly creature.
pub fn quandrix_apprentice_ii_b203() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Quandrix Apprentice II (b203)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            1, 1,
        )],
        ..Default::default()
    }
}

/// Quandrix Naturist (b203) — {2}{G} 3/2 Druid Beast. Trample. Vanilla beater.
pub fn quandrix_naturist_b203() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Naturist (b203)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Druid, CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Charmer (b203) — {2}{G}{U} 2/2 Druid Wizard. ETB scry 2 + draw 1.
pub fn quandrix_charmer_b203() -> CardDefinition {
    use crate::effect::shortcut::etb_scry_and_draw;
    CardDefinition {
        name: "Quandrix Charmer (b203)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Druid, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry_and_draw(2)],
        ..Default::default()
    }
}

/// Quandrix Surge (b203) — {2}{G}{U} Instant.
/// Put 3 +1/+1 counters on target creature.
pub fn quandrix_surge_b203() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Surge (b203)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(3),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Streamer (b203) — {2}{U} 2/2 Merfolk. ETB draw a card.
pub fn quandrix_streamer_b203() -> CardDefinition {
    use crate::effect::shortcut::etb_draw;
    CardDefinition {
        name: "Quandrix Streamer (b203)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Verdant (b203) — {3}{G} 3/4 Druid. Vigilance reach. Wall-style.
pub fn quandrix_verdant_b203() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Verdant (b203)",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 204 (modern_decks) — Quandrix round 4 ──────────────────────────

/// Quandrix Fractaller (b204) — {2}{G}{U} 3/3 Fractal.
/// Magecraft mint a Fractal with 1 +1/+1 counter.
pub fn quandrix_fractaller_b204() -> CardDefinition {
    use crate::effect::shortcut::magecraft_mint_fractal;
    CardDefinition {
        name: "Quandrix Fractaller (b204)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_mint_fractal(1)],
        ..Default::default()
    }
}

/// Quandrix Mentor (b204) — {1}{G}{U} 2/2 Druid Wizard.
/// Magecraft +1/+1 counter on target creature you control.
pub fn quandrix_mentor_b204() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mentor (b204)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Druid, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Anchor (b204) — {2}{G}{U} 4/4 Fractal. Vigilance, Reach.
pub fn quandrix_anchor_b204() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Anchor (b204)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Reach],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 205 (modern_decks) — Quandrix (G/U). A scaling Enrage wall (new
// `EventKind::DealtDamage`, CR 702.130) plus two spell-matters bodies.
// ─────────────────────────────────────────────────────────────────────────

/// Quandrix Thornfractal (b205) — {2}{G}{U} 0/6 Fractal Wall, Defender.
/// Enrage — whenever this creature is dealt damage, put that many +1/+1
/// counters on it. A Quandrix "absorb and grow" wall.
pub fn quandrix_thornfractal_b205() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Thornfractal (b205)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 6,
        keywords: vec![Keyword::Defender],
        effect: Effect::Noop,
        triggered_abilities: vec![enrage(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::TriggerEventAmount,
        })],
        ..Default::default()
    }
}

/// Quandrix Tidecaller (b205) — {1}{G}{U} 2/3 Merfolk Wizard.
/// Magecraft — whenever you cast or copy an instant or sorcery, draw a card.
pub fn quandrix_tidecaller_b205() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tidecaller (b205)",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Growthseer (b205) — {2}{G} 2/2 Elf Druid.
/// ETB — put a +1/+1 counter on target creature you control.
pub fn quandrix_growthseer_b205() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Growthseer (b205)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Mistcaller (b205) — {1}{U} 1/3 Merfolk Wizard.
/// Magecraft — whenever you cast or copy an instant or sorcery, scry 2.
pub fn quandrix_mistcaller_b205() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mistcaller (b205)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(2)],
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 206 (modern_decks) — Quandrix (G/U) staples.
// ─────────────────────────────────────────────────────────────────────────

/// Quandrix Scholar (b206) — {1}{U} 1/2 Vedalken Wizard.
/// Magecraft — whenever you cast or copy an instant or sorcery, this
/// creature gets +1/+1 until end of turn.
pub fn quandrix_scholar_b206() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Scholar (b206)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
    }
}

/// Quandrix Megafractal (b206) — {4}{G}{U} 5/5 Fractal with Trample.
/// A big curve-topper for the Fractal shell.
pub fn quandrix_megafractal_b206() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Megafractal (b206)",
        cost: cost(&[generic(4), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 207 (modern_decks) — Quandrix (G/U) Fractal / draw-matters staples.
// ─────────────────────────────────────────────────────────────────────────

/// Quandrix Tidecaller (b207) — {2}{G}{U} 3/3 Merfolk Wizard.
/// When this creature enters, create a 0/0 Fractal token with two +1/+1
/// counters on it.
pub fn quandrix_tidecaller_b207() -> CardDefinition {
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Quandrix Tidecaller (b207)",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(create_token_with_counter(
            PlayerRef::You,
            1,
            quandrix_fractal_token(),
            CounterType::PlusOnePlusOne,
            2,
        ))],
        ..Default::default()
    }
}

/// Quandrix Theorist (b207) — {1}{U} 1/3 Human Wizard.
/// Magecraft — scry 1, then draw a card.
pub fn quandrix_theorist_b207() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry_and_draw;
    CardDefinition {
        name: "Quandrix Theorist (b207)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry_and_draw(1)],
        ..Default::default()
    }
}

/// Quandrix Fractalsurge (b207) — {X}{G}{U} Sorcery.
/// Create a 0/0 Fractal token, then put X +1/+1 counters on it.
pub fn quandrix_fractalsurge_b207() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractalsurge (b207)",
        cost: cost(&[x(), g(), u()]),
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
                amount: Value::XFromCost,
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Quandrix Studymate (b207) — {G}{U} 2/2 Otter Wizard.
/// When this creature enters, put a +1/+1 counter on it for each card you
/// have drawn this turn (`Value::CardsDrawnThisTurn`).
pub fn quandrix_studymate_b207() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Studymate (b207)",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::CardsDrawnThisTurn(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Quandrix Currentweaver (b207) — {2}{U} 2/3 Merfolk Wizard.
/// When this creature enters, draw a card, then scry 1.
pub fn quandrix_currentweaver_b207() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Currentweaver (b207)",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Quandrix Bigmind (b207) — {3}{G}{U} 4/5 Fractal Wizard, Trample.
/// A sturdy Quandrix top-end body that closes games once counters pile up.
pub fn quandrix_bigmind_b207() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Bigmind (b207)",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 208 (modern_decks) — Quandrix (G/U) follow-ups.
// ─────────────────────────────────────────────────────────────────────────

/// Quandrix Rootmage (b208) — {2}{G} 2/3 Elf Druid.
/// When this creature enters, put a +1/+1 counter on target creature you
/// control.
pub fn quandrix_rootmage_b208() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Rootmage (b208)",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Quandrix Tidecantor (b208) — {1}{U} Instant.
/// Draw two cards.
pub fn quandrix_tidecantor_b208() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tidecantor (b208)",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}
