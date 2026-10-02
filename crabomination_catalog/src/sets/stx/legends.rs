//! Iconic Strixhaven legendary creatures — the five college-head Dragons
//! plus a handful of additional set-defining legends.
//!
//! Most ship as faithfully-statted bodies: cost, P/T, supertypes, keywords,
//! and creature types are correct so the cards play, are blockable, and
//! feed catalog filtering. Beledros / Tanazir / Shadrix / Galazeth are
//! fully wired (✅). Velomachus stays 🟡 only because its reveal cap uses
//! the live power via `ManaValueAtMostSourcePower`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, SelectionRequirement,
    Selector, Subtypes, Supertype,
};
use crate::mana::{b, cost, g, generic, r, u, w};

// ── Galazeth Prismari (U/R) ─────────────────────────────────────────────────

/// Galazeth Prismari — {2}{U}{R}, 3/4 Legendary Elder Dragon. Real Oracle:
/// "Flying / When Galazeth Prismari enters, create a Treasure token. /
/// Artifacts you control have '{T}: Add one mana of any color. Spend this
/// mana only to cast an instant or sorcery spell.'"
///
/// Body + Flying + ETB Treasure token, plus the restricted-mana static via
/// `grant_tap_for_any_color_restricted` — the granted tap ability's mana
/// carries `SpendRestriction::InstantSorceryOnly`, matching the printed
/// "spend this mana only to cast an instant or sorcery spell" rider.
pub fn galazeth_prismari() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility, Value};
    use crate::effect::PlayerRef;
    use crate::effect::shortcut::grant_tap_for_any_color_restricted;
    use crate::mana::SpendRestriction;
    CardDefinition {
        name: "Galazeth Prismari",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elder, CreatureType::Dragon],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        // ETB: create a Treasure token.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: std::sync::Arc::new(crate::game::effects::treasure_token()),
            },
        }],
        // "Artifacts you control have '{T}: Add one mana of any color.
        // Spend this mana only to cast an instant or sorcery spell.'"
        static_abilities: vec![grant_tap_for_any_color_restricted(
            SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
            1,
            SpendRestriction::InstantSorceryOnly,
        )],
        ..Default::default()
    }
}

// ── Beledros Witherbloom (B/G) ─────────────────────────────────────────────

/// Beledros Witherbloom — {5}{B}{G}, 4/4 Legendary Elder Dragon. Real
/// Oracle: "Flying / At the beginning of each upkeep, create a 1/1 black
/// and green Pest creature token with 'When this token dies, you gain 1
/// life.' / Pay 10 life: Untap all lands you control. Activate only once
/// each turn."
///
/// Fully wired: the upkeep trigger is `StepBegins(Upkeep)` scoped
/// `AnyPlayer` (fires on EVERY upkeep, yours and each opponent's) minting
/// the shared `stx_pest_token`. The activation uses `life_cost: 10` +
/// `once_per_turn: true` (printed "Activate only once each turn" — NOT a
/// sorcery-speed gate) + `Effect::Untap` over each land you control.
pub fn beledros_witherbloom() -> CardDefinition {
    use crate::card::{
        ActivatedAbility, EventKind, EventScope, EventSpec, SelectionRequirement, TriggeredAbility,
        Value,
    };
    use crate::effect::{PlayerRef, Selector};
    use crate::game::types::TurnStep;
    use crate::mana::ManaCost;
    CardDefinition {
        name: "Beledros Witherbloom",
        cost: cost(&[generic(5), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elder, CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        // "At the beginning of each upkeep, create a 1/1 black and green
        // Pest creature token with 'When this token dies, you gain 1 life.'"
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: std::sync::Arc::new(super::shared::stx_pest_token()),
            },
        }],
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
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 10,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None,
            from_hand: false,
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Velomachus Lorehold (R/W) ──────────────────────────────────────────────

/// Velomachus Lorehold — {5}{R}{W}, 5/5 Legendary Elder Dragon. Real
/// Oracle: "Flying, vigilance, haste / Whenever Velomachus Lorehold
/// attacks, look at the top seven cards of your library. You may cast an
/// instant or sorcery spell with mana value less than or equal to
/// Velomachus Lorehold's power from among them without paying its mana
/// cost. Put the rest on the bottom of your library in a random order."
///
/// `RevealTopMayCastOneFree`: the controller picks among the qualifying
/// cards and casts it as the trigger resolves. The seven are revealed rather
/// than looked at, and the rest keep their order on the bottom.
pub fn velomachus_lorehold() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::Selector;
    CardDefinition {
        name: "Velomachus Lorehold",
        cost: cost(&[generic(5), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Elder],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Haste],
        // CR 601.2 — the spell is cast from among the top seven as the
        // trigger resolves (the controller's pick, MV at most the live power);
        // the rest go to the bottom.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::RevealTopMayCastOneFree {
                count: crate::card::Value::Const(7),
                max_mv: crate::card::Value::PowerOf(Box::new(Selector::This)),
                filter: Some(
                    SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
            },
        }],
        ..Default::default()
    }
}

// ── Tanazir Quandrix (G/U) ─────────────────────────────────────────────────

/// Tanazir Quandrix — {3}{G}{U}, 4/4 Legendary Elder Dragon. Real Oracle:
/// "Flying, trample / When Tanazir Quandrix enters, double the number of
/// +1/+1 counters on target creature you control. / Whenever Tanazir
/// Quandrix attacks, you may have the base power and toughness of other
/// creatures you control become equal to Tanazir Quandrix's power and
/// toughness until end of turn."
///
/// ✅ Both triggers wired:
/// * **ETB** — `DoubleCountersOnEach` against a single target creature
///   you control (honors counter-doubling replacements per CR 701.10).
/// * **Attack** — `MayDo` wrapping `SetBasePT` over each OTHER creature
///   you control, reading the live `PowerOf/ToughnessOf(This)` (layer-7b
///   base P/T override until end of turn).
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
            creature_types: vec![CreatureType::Dragon, CreatureType::Elder],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![
            // "When Tanazir Quandrix enters, double the number of +1/+1
            // counters on target creature you control."
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::DoubleCountersOnEach {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                },
            },
            // "Whenever Tanazir Quandrix attacks, you may have the base
            // power and toughness of other creatures you control become
            // equal to Tanazir Quandrix's power and toughness until end
            // of turn."
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "have the base power and toughness of other creatures \
                                  you control become equal to Tanazir Quandrix's power \
                                  and toughness until end of turn"
                        .to_string(),
                    body: Box::new(Effect::SetBasePT {
                        what: Selector::EachPermanent(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByYou)
                                .and(SelectionRequirement::OtherThanSource),
                        ),
                        power: Value::PowerOf(Box::new(Selector::This)),
                        toughness: Value::ToughnessOf(Box::new(Selector::This)),
                        duration: Duration::EndOfTurn,
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

// ── Shadrix Silverquill (W/B) ──────────────────────────────────────────────

/// Shadrix Silverquill — {3}{W}{B}, 2/5 Legendary Elder Dragon. Flying, double
/// strike. At the beginning of combat on your turn, you may choose two, each
/// mode targeting a different player: that player creates a 2/1 flying Inkling;
/// draws a card and loses 1 life; or puts a +1/+1 counter on each creature
/// they control. Each picked mode owns its own player slot; the auto-filler
/// never reuses a player (default: draw on you, Inkling on an opponent).
pub fn shadrix_silverquill() -> CardDefinition {
    use crate::card::{
        CounterType, EventKind, EventScope, EventSpec, SelectionRequirement, Selector,
        TriggeredAbility, Value,
    };
    use crate::effect::PlayerRef;
    use crate::game::types::TurnStep;
    // The STX Inkling: 2/1 white-and-black flier (the SOS token is 1/1).
    let stx_inkling = crate::card::TokenDefinition {
        name: "Inkling".to_string(),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::White, crate::mana::Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Shadrix Silverquill",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Elder],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::DoubleStrike],
        triggered_abilities: vec![TriggeredAbility {
            // "At the beginning of combat on your turn, you may choose two."
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "choose two — each mode must target a different player".to_string(),
                body: Box::new(Effect::ChooseN {
                    // Default line: draw + lose 1 (mode 1), then mint an
                    // Inkling (mode 0). Each target-bearing mode owns its
                    // own player-target slot in picks order.
                    picks: vec![1, 0],
                    modes: vec![
                        // Mode 0: Target player creates a 2/1 white and
                        // black Inkling creature token with flying.
                        Effect::CreateToken {
                            who: PlayerRef::Target(0),
                            count: Value::Const(1),
                            definition: std::sync::Arc::new(stx_inkling),
                        },
                        // Mode 1: "Target player draws a card and loses 1
                        // life." Each picked mode owns its own player slot
                        // (ChooseN rebinds each mode's slot as Target(0));
                        // the auto-filler biases this one to the controller
                        // and keeps player slots distinct across modes.
                        Effect::Seq(vec![
                            Effect::Draw {
                                who: Selector::Player(PlayerRef::Target(0)),
                                amount: Value::Const(1),
                            },
                            Effect::LoseLife {
                                who: Selector::Player(PlayerRef::Target(0)),
                                amount: Value::Const(1),
                            },
                        ]),
                        // Mode 2: Target player puts a +1/+1 counter on
                        // each creature they control.
                        Effect::AddCounter {
                            what: Selector::ControlledBy {
                                who: PlayerRef::Target(0),
                                filter: SelectionRequirement::Creature,
                            },
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::Const(1),
                        },
                    ],
                }),
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
