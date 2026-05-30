//! Iconic Strixhaven legendary creatures — the five college-head Dragons
//! plus a handful of additional set-defining legends.
//!
//! Most ship as faithfully-statted bodies: cost, P/T, supertypes, keywords,
//! and creature types are correct so the cards play, are blockable, and
//! feed catalog filtering. Push XXXV completes the Beledros / Tanazir /
//! Shadrix triggers and reshapes them into ✅ — only Galazeth (artifact
//! tap-for-any-color static) and Velomachus (reveal-and-cast-from-exile)
//! still stay 🟡 pending those engine primitives.

use super::no_abilities;
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
            },
        }],
        static_abilities: vec![grant_tap_for_any_color(
            SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
        )],
        ..Default::default()
    }
}

// ── Beledros Witherbloom (B/G) ─────────────────────────────────────────────

/// Beledros Witherbloom — {3}{B}{B}{G}{G}, 6/6 Legendary Demon. Flying,
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
        cost: cost(&[generic(3), b(), b(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
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
            tap_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
    }
}

// ── Velomachus Lorehold (R/W) ──────────────────────────────────────────────

/// Velomachus Lorehold — {3}{R}{R}{W}, 5/5 Legendary Dragon, Flying /
/// vigilance / haste. Attack trigger reveals from the top until an IS card
/// with MV ≤ 5 (printed power) is found, exiles it, and grants a free cast
/// this turn (`RevealUntilFind` + `GrantMayPlay`).
pub fn velomachus_lorehold() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::{PlayerRef, RevealMissDest, ZoneDest};
    CardDefinition {
        name: "Velomachus Lorehold",
        cost: cost(&[generic(3), r(), r(), w()]),
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
        activated_abilities: no_abilities(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
    }
}

// ── Tanazir Quandrix (G/U) ─────────────────────────────────────────────────

/// Tanazir Quandrix — {2}{G}{G}{U}{U}, 5/5 Legendary Dragon. Flying,
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
        cost: cost(&[generic(2), g(), g(), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
    }
}

// ── Shadrix Silverquill (W/B) ──────────────────────────────────────────────

/// Shadrix Silverquill — {2}{W}{B}, 4/4 Legendary Dragon. Flying, double
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
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::DoubleStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
    }
}
