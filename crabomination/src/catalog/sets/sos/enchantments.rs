//! Secrets of Strixhaven (SOS) — Enchantments.
//!
//! SOS has a small enchantment subtheme around the Repartee trigger
//! (Graduation Day) and graveyard recursion (Living History). Each card
//! is added piecemeal here as the engine acquires the primitives needed
//! to express its full Oracle text.

use super::no_abilities;
use super::sorceries::{fractal_token, spirit_token};
use crate::card::{
    CardDefinition, CardType, CounterType, Effect, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{repartee, target_filtered};
use crate::effect::{Duration, PlayerRef, Selector, Value};
use crate::mana::{cost, g, generic, r, w};

/// Graduation Day — {W} Enchantment.
/// "Repartee — Whenever you cast an instant or sorcery spell that targets
/// a creature, put a +1/+1 counter on target creature you control."
///
/// Wired entirely on existing primitives: the Repartee shortcut (instant
/// or sorcery + spell-targets-creature predicate) chained with a
/// `target_filtered` `Creature & ControlledByYou` `AddCounter` body. The
/// trigger's own creature target is auto-picked at trigger-resolution
/// time by `auto_target_for_effect`.
pub fn graduation_day() -> CardDefinition {
    CardDefinition {
        name: "Graduation Day",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![repartee(Effect::AddCounter {
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
    }
}

/// Living History — {1}{R} Enchantment.
/// "When this enchantment enters, create a 2/2 red and white Spirit
/// creature token. / Whenever you attack, if a card left your graveyard
/// this turn, target attacking creature gets +2/+0 until end of turn."
///
/// ETB Spirit token wired via the shared `spirit_token()` helper. The
/// attack-trigger fires on every Attacks-event scoped to the
/// controller, but uses the new `EventKind::CardLeftGraveyard`-backed
/// per-turn tally (`Player.cards_left_graveyard_this_turn`) to gate
/// the +2/+0 pump via `Predicate::CardsLeftGraveyardThisTurnAtLeast`.
/// Single-attacker scope: each attacker fires its own trigger and the
/// pump targets the triggering attacker (not "target" in the
/// controller-picks sense — we lean on the auto-targeter's source
/// preference to land the pump on the just-declared attacker).
pub fn living_history() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec};
    CardDefinition {
        name: "Living History",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            // ETB: create a 2/2 R/W Spirit creature token.
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: spirit_token(),
                },
            },
            // On any attack you control: if a card left your graveyard
            // this turn, +2/+0 EOT to the attacking creature
            // (TriggerSource = the just-declared attacker).
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
                effect: Effect::If {
                    cond: Predicate::CardsLeftGraveyardThisTurnAtLeast {
                        who: PlayerRef::You,
                        at_least: Value::Const(1),
                    },
                    then: Box::new(Effect::PumpPT {
                        what: Selector::TriggerSource,
                        power: Value::Const(2),
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Comforting Counsel — {1}{G} Enchantment.
/// "Whenever you gain life, put a growth counter on this enchantment.
/// As long as there are five or more growth counters on this
/// enchantment, creatures you control get +3/+3."
///
/// Approximation: the lifegain-driven growth-counter accrual is wired
/// faithfully via a `LifeGained / YourControl` trigger. The static
/// `+3/+3` anthem-while-≥5-counters is **omitted** — the engine has no
/// `StaticEffect` whose toggle is gated on the source's own counter
/// count. Tracked in TODO.md under "Self-Counter-Scaled Cost Reduction"
/// (the same shape applies). Once the gate primitive lands, restoring
/// the anthem is a one-line `static_abilities` push.
pub fn comforting_counsel() -> CardDefinition {
    CardDefinition {
        name: "Comforting Counsel",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Growth,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}


/// Primary Research — {4}{W} Enchantment.
/// "When this enchantment enters, return target nonland permanent card
/// with mana value 3 or less from your graveyard to the battlefield. /
/// At the beginning of your end step, if a card left your graveyard
/// this turn, draw a card."
///
/// Both abilities are wireable on existing primitives:
/// - ETB: `Effect::Move` from graveyard with a `Nonland & ManaValueAtMost(3)`
///   target filter, destination `Battlefield(You)` — auto-target picker
///   prefers the highest-impact eligible card.
/// - End-step draw: gated on `Predicate::CardsLeftGraveyardThisTurnAtLeast`
///   (the same per-turn tally used by Living History / Hardened Academic).
///   Triggers on `EventKind::StepBegins(TurnStep::End)` scoped to the
///   active player; the controller-of-source filter inside the trigger
///   resolution path picks the right player to draw.
pub fn primary_research() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Primary Research",
        cost: cost(&[generic(4), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            // ETB: return ≤MV3 nonland permanent card from your gy → bf.
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Nonland
                            .and(SelectionRequirement::ManaValueAtMost(3)),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            },
            // Your end step: if a card left your gy this turn, draw.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::CardsLeftGraveyardThisTurnAtLeast {
                        who: PlayerRef::You,
                        at_least: Value::Const(1),
                    },
                    then: Box::new(Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Additive Evolution — {3}{G}{G} Enchantment.
/// "When this enchantment enters, create a 0/0 green and blue Fractal
/// creature token. Put three +1/+1 counters on it. / At the beginning of
/// combat on your turn, put a +1/+1 counter on target creature you
/// control. It gains vigilance until end of turn."
///
/// ETB Fractal-with-3-counters wired via the existing `fractal_token()`
/// helper + `Selector::LastCreatedToken` (so the AddCounter sticks to
/// the freshly-minted token, not a random board piece). Begin-combat
/// pump+vigilance on a friendly creature wired with a `target_filtered`
/// `Creature & ControlledByYou` slot — the auto-target picker prefers
/// the highest-power friendly creature, biasing the buff toward the
/// active threat.
pub fn additive_evolution() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Additive Evolution",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            // ETB: create a 0/0 green-blue Fractal token + 3 +1/+1 counters.
            TriggeredAbility {
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
                        amount: Value::Const(3),
                    },
                ]),
            },
            // Begin combat (active player only): +1/+1 counter + vigilance EOT.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByYou),
                        ),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Vigilance,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}
