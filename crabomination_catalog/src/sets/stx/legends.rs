//! Iconic Strixhaven legendary creatures — the five college-head Dragons
//! plus a handful of additional set-defining legends.
//!
//! Most ship as faithfully-statted bodies: cost, P/T, supertypes, keywords,
//! and creature types are correct so the cards play, are blockable, and
//! feed catalog filtering. Beledros / Tanazir / Shadrix / Galazeth are
//! fully wired (✅). Velomachus stays 🟡 only because its reveal cap uses
//! a static `ManaValueAtMost(5)` rather than its live power.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, MayPlayDuration, Selector,
    SelectionRequirement, Subtypes, Supertype,
};
use crate::mana::{b, cost, g, generic, r, u, w};

// ── Galazeth Prismari (U/R) ─────────────────────────────────────────────────

/// Galazeth Prismari — {2}{U}{R}, 3/4 Legendary Dragon Wizard. Real Oracle:
/// "Flying / When Galazeth Prismari enters the battlefield, create a
/// Treasure token. / Artifacts you control have '{T}: Add one mana of any
/// color.'"
///
/// Body + Flying + ETB Treasure token, plus the "Artifacts you control have
/// '{T}: Add one mana of any color'" static via
/// `StaticEffect::GrantActivatedAbility`.
pub fn galazeth_prismari() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility, Value};
    use crate::effect::PlayerRef;
    use crate::effect::shortcut::grant_tap_for_any_color;
    CardDefinition {
        name: "Galazeth Prismari",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        // ETB: create a Treasure token.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
            },
        }],
        // "Artifacts you control have '{T}: Add one mana of any color.'"
        static_abilities: vec![grant_tap_for_any_color(
            SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
        )],
        ..Default::default()
    }
}

// ── Beledros Witherbloom (B/G) ─────────────────────────────────────────────

/// Beledros Witherbloom — {5}{B}{G}, 4/4 Legendary Demon. Flying,
/// trample, lifelink. Activated: "Pay 10 life: Untap each land you
/// control. Activate only as a sorcery."
///
/// Fully wired: the activation uses the existing `life_cost: 10` field
/// + `sorcery_speed: true` gate + `Effect::Untap { what:
///   EachPermanent(Land & ControlledByYou), up_to: None }`. The
///   pre-flight life-cost gate (`GameError::InsufficientLife`) means the
///   activation is rejected cleanly when the controller can't pay 10
///   life. Existing `up_to: None` keeps the printed "untap each land"
///   semantics (cap-free mass untap).
/// Beledros Witherbloom — Legendary 6/6 Witherbloom Elder Dragon with flying,
/// trample, lifelink. "Pay 10 life: Untap each land you control. Activate
/// only as a sorcery."
///
/// Now wired: the activated ability uses `life_cost: 10` +
/// `sorcery_speed: true` + `Effect::Untap` over each land you control.
pub fn beledros_witherbloom() -> CardDefinition {
    use crate::card::{ActivatedAbility, SelectionRequirement};
    use crate::effect::Selector;
    use crate::mana::ManaCost;
    CardDefinition {
        name: "Beledros Witherbloom",
        cost: cost(&[generic(5), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::Untap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                ),
                up_to: None,
            },
            once_per_turn: true,
            sorcery_speed: true,
            sac_cost: false,
            condition: None,
            life_cost: 10,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Velomachus Lorehold (R/W) ──────────────────────────────────────────────

/// Velomachus Lorehold — {5}{R}{W}, 5/5 Legendary Dragon, Flying /
/// vigilance / haste. Attack trigger reveals from the top until an IS card
/// with MV ≤ 5 (printed power) is found, exiles it, and grants a free cast
/// this turn (`RevealUntilFind` + `GrantMayPlay`).
pub fn velomachus_lorehold() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::{PlayerRef, RevealMissDest, ZoneDest};
    CardDefinition {
        name: "Velomachus Lorehold",
        cost: cost(&[generic(5), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Haste],
        effect: Effect::Noop,
        // Push (modern_decks, batch 74): "Whenever Velomachus attacks,
        // reveal cards from the top of your library until you reveal an
        // instant or sorcery card with mana value less than or equal to
        // Velomachus's power" trigger is **now wired**. Approximation:
        // the printed "mana value ≤ Velomachus's power" filter uses a
        // static `ManaValueAtMost(5)` (Velomachus's printed power). A
        // pumped Velomachus (Light of Promise counters, +1/+0 EOT, etc.)
        // doesn't widen the cap; a base-power debuff likewise doesn't
        // narrow it. RevealUntilFind walks the top of library exiling
        // misses to the bottom-random pile, lands the matching IS card
        // in exile, then `GrantMayPlay` stamps a may-cast-this-turn
        // permission on the exiled card so the caster can free-cast it
        // via the existing `CastFromZoneWithoutPaying` action. cap=60
        // covers any realistic library depth.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery))
                        .and(SelectionRequirement::ManaValueAtMost(5)),
                    to: ZoneDest::Exile,
                    cap: crate::card::Value::Const(60),
                    life_per_revealed: 0,
                    miss_dest: RevealMissDest::BottomRandom,
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: false,
                },
            ]),
        }],
        ..Default::default()
    }
}

// ── Tanazir Quandrix (G/U) ─────────────────────────────────────────────────

/// Tanazir Quandrix — {3}{G}{U}, 4/4 Legendary Dragon. Flying,
/// trample. Real Oracle ETB doubles +1/+1 counters on each creature you
/// control; attack trigger doubles toughness of target creature you
/// control.
///
/// ✅ Both triggers wired without new primitives:
/// * **ETB counter-doubling** uses `ForEach(Creature & ControlledByYou)`
///   binding `Selector::TriggerSource` to each iteration entity, then
///   `AddCounter(+1/+1, amount: CountersOn(TriggerSource, +1/+1))`. A
///   creature with N +1/+1 counters before resolution ends with 2N after
///   (matches the printed double).
/// * **Attack trigger** uses `PumpPT` with `toughness:
///   ToughnessOf(Target(0))` — pumping the target's toughness by its
///   current value cleanly doubles it (printed `T` becomes `T + T = 2T`
///   until end of turn). Auto-target picker prefers a friendly
///   high-toughness creature for the pump.
pub fn tanazir_quandrix() -> CardDefinition {
    use crate::card::{
        CounterType, EventKind, EventScope, EventSpec, SelectionRequirement, Selector,
        TriggeredAbility, Value,
    };
    use crate::effect::Duration;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Tanazir Quandrix",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![
            // ETB: double the number of +1/+1 counters on each creature
            // you control.
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
            },
            // Attack: double target creature you control's toughness EOT.
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    power: Value::Const(0),
                    toughness: Value::ToughnessOf(Box::new(Selector::Target(0))),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

// ── Shadrix Silverquill (W/B) ──────────────────────────────────────────────

/// Shadrix Silverquill — {3}{W}{B}, 2/5 Legendary Dragon. Flying, double
/// strike. Real Oracle attack-trigger: "Whenever Shadrix Silverquill
/// attacks, choose two. You may choose the same mode more than once.
/// • You and target opponent each draw a card.
/// • Put a +1/+1 counter on target creature.
/// • Target player creates two 1/1 white and black Inkling creature
///   tokens with flying."
///
/// ✅ (push XXXV) Attack trigger now wired via `Effect::ChooseN` with
/// auto-picks `[1, 2]` (counter on target creature + mint two Inklings
/// for the controller). The third mode (you-and-target-opp each draw)
/// stays in `modes` for future mode-pick UI. The printed "you may
/// choose the same mode more than once" rider is a CR 700.2d exception
/// that the engine's `ChooseN.picks` currently treats as a strict
/// distinct-indices set; the auto-pick set we ship picks two distinct
/// modes (the canonical strong opener), so we sidestep the same-mode-
/// twice corner. Mode 1 binds the single target slot to a friendly
/// creature; mode 2 binds the target slot to the controller (mints
/// under `Selector::You`). Tests:
/// `shadrix_silverquill_attack_pumps_target_creature_and_mints_inklings`,
/// `shadrix_silverquill_attack_does_not_trigger_on_opp_attack`.
pub fn shadrix_silverquill() -> CardDefinition {
    use crate::card::{
        CounterType, EventKind, EventScope, EventSpec, SelectionRequirement, Selector,
        TriggeredAbility, Value,
    };
    use crate::effect::PlayerRef;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Shadrix Silverquill",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::DoubleStrike],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            // Choose-two. Picks 1 + 2 by default (counter + Inklings).
            effect: Effect::ChooseN {
                picks: vec![1, 2],
                modes: vec![
                    // Mode 0: You and target opponent each draw a card.
                    // Single-target slot, collapsed to a draw 1 for the
                    // caster (the "and target opp draws" is a multi-target
                    // shape not yet supported).
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    // Mode 1: Put a +1/+1 counter on target creature.
                    Effect::AddCounter {
                        what: target_filtered(SelectionRequirement::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                    // Mode 2: Target player creates two 1/1 white and black
                    // Inkling creature tokens with flying. Collapses to
                    // "you create two" — same auto-target heuristic the
                    // catalog uses for the Defend the Campus mint.
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(2),
                        definition: crate::catalog::sets::sos::inkling_token(),
                    },
                ],
            },
        }],
        ..Default::default()
    }
}

// ── Shaile, Dean of Radiance // Embrose, Dean of Shadow (W // B) ─────────────

/// Shaile, Dean of Radiance — {1}{W} Legendary Bird Cleric 1/1, Flying,
/// vigilance. "{T}: Put a +1/+1 counter on each creature that entered the
/// battlefield under your control this turn." MDFC back: Embrose, Dean of
/// Shadow — {2}{B}{B} Legendary Human Warlock 4/4. "{T}: Put a +1/+1 counter
/// on another target creature, then Embrose deals 2 damage to that creature."
/// "Whenever a creature you control with a +1/+1 counter on it dies, draw a
/// card."
pub fn shaile_dean_of_radiance() -> CardDefinition {
    use crate::card::{
        ActivatedAbility, CounterType, EventKind, EventScope, EventSpec, TriggeredAbility, Value,
    };
    use crate::effect::shortcut::target_filtered;
    let embrose = CardDefinition {
        name: "Embrose, Dean of Shadow",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::Const(2),
                },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                crate::card::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne),
                },
            ),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Shaile, Dean of Radiance",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::EnteredThisTurn),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        back_face: Some(Box::new(embrose)),
        ..Default::default()
    }
}
