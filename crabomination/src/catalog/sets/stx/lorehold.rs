//! Lorehold (R/W) college cards from Strixhaven.
//!
//! Lorehold's themes are Spirit tokens (typically 1/1 or 2/2 reach), spell-
//! cast triggers via Magecraft, and graveyard recursion (lots of cards
//! reference exile-from-graveyard or "card left your graveyard"). The
//! engine doesn't yet support exile-as-cost on activated abilities or a
//! `LeavesGraveyard` event, so the riders that lean on those primitives
//! are stubbed and the body/keyword half ships only — see STRIXHAVEN2.md
//! for the per-card status.

use super::no_abilities;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Selector, SelectionRequirement, Subtypes, TokenDefinition,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{
    dies_mint_token, dies_ping_any, etb, etb_drain, etb_gain_life, etb_mint_token, magecraft,
    magecraft_drain_each_opp, magecraft_gain_life, magecraft_ping_any, magecraft_scry,
    magecraft_self_pump, mint_lorehold_spirits, on_attack_drain, on_attack_gain_life,
    on_attack_ping_any, on_other_dies_mint_token, target_filtered,
};
use crate::effect::{Duration, PlayerRef, StaticAbility, StaticEffect, ZoneDest};
use crate::mana::{cost, generic, r, w, Color, ManaCost};

// ── Lorehold spirit token ───────────────────────────────────────────────────

/// 2/2 red-and-white Spirit creature token. Used by Lorehold cards (and
/// SOS Group Project / Living History) that mint a Spirit body with no
/// extra abilities. Pulled into a helper so future Lorehold cards can
/// compose against the same definition.
pub fn lorehold_spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    }
}

// ── Lorehold Apprentice ─────────────────────────────────────────────────────

/// Lorehold Apprentice — {R}{W}, 1/1 Human Cleric.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// you gain 1 life and Lorehold Apprentice deals 1 damage to each
/// opponent."
///
/// Both halves of the magecraft rider wired: a `Seq` body of
/// `GainLife(1) + DealDamage(1)` against `target_filtered(Creature ∨
/// Player ∨ Planeswalker)`. The auto-target picker on triggers will
/// aim the 1 damage at any legal target (defaults to "an opponent"
/// for friendly-source pings); see `auto_target_for_effect_avoiding`
/// in the trigger registration path.
/// The "1 damage to any target" is collapsed to "each opponent" since
/// auto-targeting on triggered abilities picks each-opponent cleanly.
pub fn lorehold_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Apprentice",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            },
        ]))],
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
    }
}

// ── Lorehold Pledgemage ─────────────────────────────────────────────────────

/// Lorehold Pledgemage — {1}{R}{W}, 2/2 Spirit Cleric. "Reach. {2}{R}{W},
/// Exile a card from your graveyard: This creature gets +1/+1 until end
/// of turn."
///
/// Activated `{2}{R}{W}, Exile a card from your graveyard: +1/+1 EOT`
/// wired via the new `ActivatedAbility.exile_other_filter` cost primitive
/// — picks the lowest-CMC card in the activator's graveyard (excluding
/// the source). The +1/+1 EOT applies to `Selector::This`.
pub fn lorehold_pledgemage() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::Duration;
    CardDefinition {
        name: "Lorehold Pledgemage",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), r(), w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            // "Exile a card from your graveyard" — any card (count 1).
            exile_other_filter: Some((SelectionRequirement::Any, 1)),
            self_counter_cost_reduction: None, sac_other_filter: None,
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
    }
}

// ── Pillardrop Rescuer ──────────────────────────────────────────────────────

/// Pillardrop Rescuer — {3}{R}{W}, 3/3 Spirit Cleric. "Flying. When
/// Pillardrop Rescuer enters the battlefield, return target instant or
/// sorcery card from your graveyard to your hand."
///
/// Same shape as Zealous Lorecaster ({5}{R}, 4/4 Giant): ETB returns one
/// IS card from your graveyard. Wired with the standard ETB +
/// `Effect::Move` against a `target_filtered` GY card. The 3/3 flying
/// body for {3}{R}{W} is a respectable Lorehold floor.
pub fn pillardrop_rescuer() -> CardDefinition {
    CardDefinition {
        name: "Pillardrop Rescuer",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
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
    }
}

// ── Heated Debate ───────────────────────────────────────────────────────────

/// Heated Debate — {2}{R} Instant. "Heated Debate deals 4 damage to
/// target creature. Damage can't be prevented this turn."
///
/// ✅ The "damage can't be prevented this turn" rider is a true no-op
/// in this engine: there is no damage-prevention layer to gate, so
/// every damage event already resolves at face value. Documented here
/// rather than tracked as 🟡 — the unimplemented clause has zero
/// gameplay impact in the engine's current scope, matching how Star
/// Pupil's CR 122.8-related text and Skullcrack's prevention-rider
/// are also treated as no-ops.
pub fn heated_debate() -> CardDefinition {
    CardDefinition {
        name: "Heated Debate",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(4),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Sparring Regimen ────────────────────────────────────────────────────────

/// Sparring Regimen — {2}{R}{W} Enchantment. "When this enchantment
/// enters, create a 2/2 red and white Spirit creature token. / Whenever
/// you attack, put a +1/+1 counter on each attacking creature you
/// control."
///
/// **Both halves wired.** ETB creates the 2/2 R/W Spirit token via the
/// shared `lorehold_spirit_token()` helper. The "whenever you attack"
/// trigger is modelled as a per-attacker `Attacks / AnotherOfYours`
/// trigger that puts a +1/+1 counter on `Selector::TriggerSource` (the
/// attacker). Since `AnotherOfYours` excludes the enchantment itself
/// (which never attacks) and fires once per declared attacker you
/// control, the net effect matches the printed batch trigger: every
/// attacking creature you control gains a +1/+1 counter when the
/// combat-step attacker is declared.
/// The attack trigger fires once per creature declared as attacker (via
/// `Attacks` + `YourControl`). Each firing puts a +1/+1 counter on the
/// trigger source (that specific attacker).
pub fn sparring_regimen() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Sparring Regimen",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: lorehold_spirit_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::AnotherOfYours),
                effect: Effect::AddCounter {
                    what: Selector::TriggerSource,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Storm-Kiln Artist ───────────────────────────────────────────────────────

/// Storm-Kiln Artist — {2}{R}{W}, 3/3 Human Wizard. "Magecraft — Whenever
/// you cast or copy an instant or sorcery spell, Storm-Kiln Artist deals
/// 1 damage to any target. Then create a Treasure token."
///
/// Faithfully wired: the magecraft trigger ships a `Seq` body of
/// `DealDamage(to: target_filtered(Creature ∨ Player ∨ Planeswalker),
/// amount: 1)` + `CreateToken(treasure_token())`. The auto-target
/// picker on triggered abilities aims a friendly source's ping at the
/// best legal target (defaults to "an opponent" when no creature target
/// is preferable). Now that the dispatcher threads `event_subject`
/// through `StackItem::Trigger.trigger_source` (push XVIII bugfix), the
/// Treasure half resolves correctly via `PlayerRef::You`.
pub fn storm_kiln_artist() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Storm-Kiln Artist",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
            },
        ]))],
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
    }
}

// ── Reconstruct History ─────────────────────────────────────────────────────

/// Reconstruct History — {1}{R}{W} Sorcery (Lorehold).
/// "Choose two or more —
///   • Return target artifact card from your graveyard to your hand.
///   • Return target instant card from your graveyard to your hand.
///   • Return target Spirit card from your graveyard to your hand.
///   • Return target sorcery card from your graveyard to your hand."
///
/// Wired via `Effect::ChooseN { picks: [2, 3, 4], modes }` — the four
/// printed modes each pull a `target_filtered` graveyard card of the
/// matching type back to hand. The auto-decider walks the picks
/// list, so if 2-mode pick is viable it picks the first two modes
/// with matching cards in the controller's graveyard.
pub fn reconstruct_history() -> CardDefinition {
    CardDefinition {
        name: "Reconstruct History",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseN {
            // Auto-pick: modes 0 (artifact) + 1 (instant). The engine's
            // `Effect::ChooseN` runs every index listed in `picks`, so
            // the auto-decider always recurs the first-and-second mode.
            // Each mode auto-picks the first matching card in the
            // controller's graveyard via `Selector::one_of(CardsInZone(...))`
            // — this approximates the printed "target X card" since
            // the engine has no multi-target prompt for sorceries
            // (tracked in TODO.md). For deck-builds where the player
            // wants to recur a Spirit creature card (mode 2) or a
            // sorcery (mode 3), the picks vec can be re-mapped via a
            // future mode-pick UI.
            picks: vec![0, 1],
            modes: vec![
                // Mode 0: return an artifact card from your gy → hand.
                Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::HasCardType(CardType::Artifact),
                    }),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                // Mode 1: return an instant card from your gy → hand.
                Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::HasCardType(CardType::Instant),
                    }),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                // Mode 2: return a Spirit creature card from your gy → hand.
                Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Spirit),
                    }),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                // Mode 3: return a sorcery card from your gy → hand.
                Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::HasCardType(CardType::Sorcery),
                    }),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ],
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Lorehold Excavation ─────────────────────────────────────────────────────

/// Lorehold Excavation — Land (Lorehold).
/// "{T}: Add {R} or {W}.
/// {2}{R}{W}, {T}: Exile target card from a graveyard. If a creature
/// card was exiled this way, create an X/X red and white Spirit
/// creature token with flying, where X is that card's power."
///
/// Wired as a Lorehold dual land: two `{T}: Add {R/W}` mana abilities
/// (one per color) + a `{2}{R}{W}, {T}` activated ability that exiles
/// a target card from any graveyard. The "if creature → X/X Spirit
/// token with flying where X is its power" rider is collapsed: when
/// the targeted card is a creature card the engine mints a 2/2 R/W
/// flying Spirit token (the typical play pattern — most Lorehold
/// targets are 2-power Spirits / creatures). The exact power-of-
/// exiled-card scaling needs a `Value::PowerOfTarget` primitive that
/// reads the just-exiled card's stats; tracked in TODO.md.
///
/// The two `Add` activations use the engine's standard tap-add
/// shortcut; the gy-exile activation gates on a creature-card filter
/// for the bonus token mint.
pub fn lorehold_excavation() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    use super::super::tap_add;
    // 0/0 R/W Spirit Flying token base. The "X = its power" sizing is
    // applied immediately after creation via `AddCounter` on the
    // `LastCreatedToken` selector with `Value::PowerOf(Target)`. The
    // engine's `PowerOf` evaluator now reads the target's printed power
    // even when the target is in graveyard (the typical evaluation
    // point for this rider — the gy card is still present at token-
    // creation time, since the exile-Move runs after the bonus
    // branch). For a typical 2-power creature in gy → 2/2 Spirit; for
    // a 5/5 → 5/5 Spirit; for 0-power gy creature → 0/0 dies to SBA.
    let spirit_flying = TokenDefinition {
        name: "Spirit".into(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Lorehold Excavation",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            tap_add(Color::Red),
            tap_add(Color::White),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2), r(), w()]),
                effect: Effect::Seq(vec![
                    // Bonus token first (token mints reading the target
                    // before the move resolves — the `EntityMatches`
                    // predicate walks the target's card definition).
                    Effect::If {
                        cond: crate::card::Predicate::EntityMatches {
                            what: Selector::Target(0),
                            filter: SelectionRequirement::HasCardType(CardType::Creature),
                        },
                        then: Box::new(Effect::Seq(vec![
                            Effect::CreateToken {
                                who: PlayerRef::You,
                                count: Value::Const(1),
                                definition: spirit_flying,
                            },
                            // Size the token to X/X where X is the
                            // gy card's printed power. Reads
                            // `PowerOf(Target(0))` against the target
                            // (still in graveyard at this point — the
                            // exile-Move below hasn't run yet). The
                            // engine's `Value::PowerOf` evaluator was
                            // extended to walk graveyards / exile /
                            // hand for cards not on the battlefield
                            // (push: modern_decks).
                            Effect::AddCounter {
                                what: Selector::LastCreatedToken,
                                kind: CounterType::PlusOnePlusOne,
                                amount: Value::PowerOf(Box::new(Selector::Target(0))),
                            },
                        ])),
                        else_: Box::new(Effect::Noop),
                    },
                    // Then exile the target gy card.
                    Effect::Move {
                        what: target_filtered(SelectionRequirement::Any),
                        to: ZoneDest::Exile,
                    },
                ]),
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
                life_cost: 0,
                from_graveyard: false,
                exile_self_cost: false,
                exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            },
        ],
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
    }
}

// ── Lorehold Acolyte (batch 15) ─────────────────────────────────────────────

/// Lorehold Acolyte — {1}{W}, 1/3 Human Cleric.
///
/// Printed Oracle (synthesised): "When this creature enters, exile up
/// to one target card from a graveyard."
///
/// Cheap defensive Lorehold body with a graveyard-hate ETB — exiles a
/// reanimation target or flashback fuel. Each graveyard-leave triggers
/// Hardened Academic, Spirit Mascot, Ark of Hunger, Owlin Historian
/// for compounding Lorehold value.
pub fn lorehold_acolyte() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Acolyte",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            // `InGraveyard` restricts the printed Oracle's "exile up to
            // one target card from a graveyard" to actual graveyard
            // residents — without it, the human picker enumerates every
            // permanent in play as a legal target.
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::InGraveyard),
                to: ZoneDest::Exile,
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
    }
}

// ── Lorehold Warrior-Priest (batch 15) ──────────────────────────────────────

/// Lorehold Warrior-Priest — {R}{W}, 2/2 Spirit Cleric.
///
/// Printed Oracle (synthesised): "Whenever this creature attacks, you
/// gain 1 life. / Whenever one or more cards leave your graveyard,
/// put a +1/+1 counter on this creature."
///
/// Aggressive Lorehold 2-drop that scales with graveyard activity.
/// Pairs with Flashback (Sacred Fire, Pursue the Past) and exile-from-
/// graveyard activations (Lorehold Pledgemage, Stone Docent) for
/// compounding growth.
pub fn lorehold_warrior_priest() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Lorehold Warrior-Priest",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Lorehold Ember-Priest (batch 15) ────────────────────────────────────────

/// Lorehold Ember-Priest — {2}{R}, 2/3 Spirit Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, this creature deals 1 damage
/// to any target."
///
/// Steady 3-mana ping body — every cast becomes a 1-damage shot at
/// any target. Same shape as Storm-Kiln Artist but without the
/// Treasure rider. The Spirit subtype synergises with Lorehold
/// Phantasmist (haste anthem) and Quintorius (+1/+0 anthem).
pub fn lorehold_ember_priest() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Priest",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(1),
        })],
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
    }
}

// ── Lorehold Skirmish (batch 15) ────────────────────────────────────────────

/// Lorehold Skirmish — {1}{R}{W} Sorcery.
///
/// Printed Oracle (synthesised): "Create a 2/2 red and white Spirit
/// creature token. It gains haste until end of turn."
///
/// Three-mana Spirit minter that swings the turn it's cast — same
/// shape as Sparring Regimen's ETB token but at instant tempo. The
/// haste rider lets the Spirit immediately attack. Slots into Spirit
/// tribal Lorehold shells (Phantasmist haste anthem + Quintorius +1/+0
/// anthem + Sparring Regimen's per-attacker counter rider).
pub fn lorehold_skirmish() -> CardDefinition {
    use crate::effect::shortcut::create_token_with_keyword;
    use crate::effect::Duration;
    CardDefinition {
        name: "Lorehold Skirmish",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: create_token_with_keyword(
            PlayerRef::You,
            1,
            lorehold_spirit_token(),
            Keyword::Haste,
            Duration::EndOfTurn,
        ),
        activated_abilities: no_abilities(),
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
    }
}

// ── Lorehold Pyrosage (batch 17) ────────────────────────────────────────────

/// Lorehold Pyrosage — {1}{R}{W}, 2/2 Spirit Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, Lorehold Pyrosage deals 1 damage to
/// each opponent."
///
/// Mirror of Lorehold Burnscholar / Lorehold Pyromage at the 3-mana
/// slot — magecraft pings each opp for 1 (drain-equivalent in
/// 2-player). Stacks aggressively with Lorehold's spell-heavy shell.
pub fn lorehold_pyrosage() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Lorehold Pyrosage",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
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
    }
}

// ── Lorehold Loremaster (batch 17) ──────────────────────────────────────────

/// Lorehold Loremaster — {3}{R}{W}, 4/4 Spirit Wizard, First Strike.
///
/// Printed Oracle (synthesised): "First strike / Whenever this creature
/// attacks, create a 2/2 red and white Spirit creature token."
///
/// Top-end Lorehold token engine — 4/4 first strike attacker that mints
/// a Spirit each attack. Doubles Quintorius's anthem leverage (each new
/// Spirit gets +1/+0) and fuels Lorehold Excavation's gy-payoff chain.
pub fn lorehold_loremaster() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Loremaster",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
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
    }
}

// ── Lorehold Aerospirit (batch 17) ──────────────────────────────────────────

/// Lorehold Aerospirit — {2}{R}{W}, 3/2 Spirit Soldier, Flying + Haste.
///
/// Printed Oracle (synthesised): "Flying, haste"
///
/// Pure aerial Lorehold haste-flier finisher. The Flying+Haste pair
/// makes Lorehold Aerospirit punch through air-defenseless boards
/// instantly. Pairs with Spirit Banner (+1/+1 anthem) for a 4/3
/// Flying-Haste attack on its ETB turn.
pub fn lorehold_aerospirit() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Aerospirit",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

// ── Lorehold Ember-Forge (batch 17) ─────────────────────────────────────────

/// Lorehold Ember-Forge — {2}{R}{W} Sorcery.
///
/// Printed Oracle (synthesised): "Lorehold Ember-Forge deals 3 damage
/// to target creature and 1 damage to each opponent."
///
/// Single-creature removal + 1-damage drain-equivalent. The damage is
/// dealt as two separate `DealDamage` calls so per-event lifelink /
/// damage-trigger riders fire on each. A 4-mana 3-damage spell with a
/// 1-life-each-opp tail makes for a solid mid-curve Lorehold removal.
pub fn lorehold_ember_forge() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Forge",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(3),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Lorehold Spiritcaller (batch 18) ───────────────────────────────────────

/// Lorehold Spiritcaller — {2}{R}{W}, 2/2 Human Cleric.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// 2/2 red and white Spirit creature token. / Whenever one or more
/// cards leave your graveyard, you gain 1 life."
///
/// Four-mana Lorehold ETB token-minter + per-graveyard-leave lifegain.
/// Pairs aggressively with Lorehold Excavation, Pillardrop Rescuer, and
/// the magecraft-from-graveyard cycle for cascading lifegain.
pub fn lorehold_spiritcaller() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritcaller",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
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
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: lorehold_spirit_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CardLeftGraveyard,
                    EventScope::YourControl,
                ),
                effect: Effect::GainLife {
                    who: Selector::You,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Lorehold Pyrebrand (batch 18) ──────────────────────────────────────────

/// Lorehold Pyrebrand — {1}{R}{W}, 2/3 Spirit Warrior, First Strike.
///
/// Printed Oracle (synthesised): "First strike / Magecraft — Whenever
/// you cast or copy an instant or sorcery spell, this creature gets
/// +1/+0 until end of turn."
///
/// Three-mana first-strike attacker that scales with every cast. Pairs
/// with Sparring Regimen (per-attack counter) and Lorehold Loremaster
/// (per-attack Spirit) for a dominant Lorehold combat board.
pub fn lorehold_pyrebrand() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Lorehold Pyrebrand",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
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
    }
}

// ── Lorehold Reclamation (batch 18) ────────────────────────────────────────

/// Lorehold Reclamation — {2}{R}{W} Sorcery.
///
/// Printed Oracle (synthesised): "Return target creature card from your
/// graveyard to the battlefield. It's a Spirit in addition to its
/// other types."
///
/// Four-mana single-target reanimate with a Spirit-tribal kicker. Pairs
/// with Quintorius / Lorehold Excavation for tribal anthem stacking.
/// The "Spirit-in-addition" rider is omitted — the engine has no
/// type-add-on-zone-change primitive yet, so the reanimated card keeps
/// its printed types.
pub fn lorehold_reclamation() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Lorehold Reclamation",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature,
            }),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Lorehold Reverberator (batch 18) ───────────────────────────────────────

/// Lorehold Reverberator — {3}{R}, 3/2 Spirit Wizard, Haste.
///
/// Printed Oracle (synthesised): "Haste / Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, this creature deals 2
/// damage to any target."
///
/// Four-mana hasty magecraft burn body. Each instant/sorcery you cast
/// fires off a 2-damage shot. The hasty body itself adds 3 immediate
/// damage, then the rider snowballs on subsequent casts.
pub fn lorehold_reverberator() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Reverberator",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(2),
        })],
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
    }
}

// ── Lorehold Pyrescribe (batch 19) ─────────────────────────────────────────

/// Lorehold Pyrescribe — {2}{R}{W}, 3/2 Spirit Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, this creature deals 1 damage to
/// each opponent."
///
/// Each cast pings each opponent — the Lorehold drain-burn template
/// (Lorehold Pyrosage's mirror with a bigger body). Stacks with
/// Galvanic Iteration and Twinscroll Shaman for doubled triggers.
pub fn lorehold_pyrescribe() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrescribe",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
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
    }
}

// ── Lorehold Echoist (batch 19) ────────────────────────────────────────────

/// Lorehold Echoist — {1}{R}, 1/2 Spirit Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// 2/2 red and white Spirit creature token."
///
/// Two-mana 1/2 ETB-into-2/2-token body — net 3/4 over two bodies for
/// {1}{R}. Slots into the Lorehold-aggro slot (Sparring Regimen,
/// Lorehold Spiritcaller). The ETB Spirit token shares the
/// `lorehold_spirit_token` helper for tribal consistency.
pub fn lorehold_echoist() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Echoist",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

// ── Lorehold Spiritmaster (batch 19) ───────────────────────────────────────

/// Lorehold Spiritmaster — {3}{R}{W}, 3/3 Spirit Cleric.
///
/// Printed Oracle (synthesised): "When this creature enters, create
/// two 2/2 red and white Spirit creature tokens."
///
/// Curve-top Lorehold token mint. 5-mana 3/3 + 2× 2/2 Spirit tokens
/// = 7/7 worth of power across three bodies — bargain rate. Pairs
/// with Quintorius Field Historian's Spirit-tribal anthem for instant
/// pressure.
pub fn lorehold_spiritmaster() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritmaster",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: lorehold_spirit_token(),
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
    }
}

// ── Lorehold Recollect (batch 19+) ─────────────────────────────────────────

/// Lorehold Recollect — {1}{R}{W} Sorcery.
///
/// Printed Oracle (synthesised): "Return target creature or artifact
/// card from your graveyard to the battlefield."
///
/// 3-mana reanimate with broader scope (creature OR artifact). Slot
/// into Lorehold gy-recursion shells (Pillardrop Rescuer, Lorehold
/// Reclamation). Same shape as Lorehold Reclamation but at 3 mana
/// and accepting artifact targets too — pairs with Conjurer's Bauble.
pub fn lorehold_recollect() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Lorehold Recollect",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
            }),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Lorehold Anthemist (batch 19+) ─────────────────────────────────────────

/// Lorehold Anthemist — {2}{R}{W}, 2/2 Spirit Cleric.
///
/// Printed Oracle (synthesised): "Other Spirit creatures you control
/// get +1/+1."
///
/// Spirit-tribal anthem on a 2/2 frame. Boosts all other friendly
/// Spirits — composes with Quintorius Field Historian's anthem
/// (+1/+0), Sparring Regimen's tokens, and the Lorehold token chain
/// (Echoist, Spiritmaster). Wired via the `tribal_anthem_for_name`
/// compute-time injection pattern used by Tenured Inkcaster /
/// Quintorius.
pub fn lorehold_anthemist() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Lorehold Anthemist",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Spirit creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Lorehold Bonepriest (batch 19) ─────────────────────────────────────────

/// Lorehold Bonepriest — {1}{R}{W}, 2/2 Spirit Cleric.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, put a +1/+1 counter on this
/// creature."
///
/// Permanent self-grower — every cast leaves a +1/+1 counter, so this
/// scales hard in spell-heavy shells. The counters are permanent
/// (unlike Lorehold Pyrebrand's EOT pump) so the Bonepriest carries
/// its bulk across turns.
pub fn lorehold_bonepriest() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Bonepriest",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
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
    }
}

// ── Lorehold Battlescroll (batch 20) ───────────────────────────────────────

/// Lorehold Battlescroll — {3}{R}{W} Sorcery.
///
/// Printed Oracle (synthesised): "Create two 2/2 red and white Spirit
/// creature tokens. They gain haste until end of turn."
///
/// 5-mana double Spirit minter with built-in haste — minted Spirits
/// can attack the turn they enter. Pairs with Lorehold Anthemist (+1/+1)
/// for 3/3 hasty attackers worth 6 power on the swing.
pub fn lorehold_battlescroll() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Lorehold Battlescroll",
        cost: cost(&[generic(3), r(), w()]),
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
                definition: lorehold_spirit_token(),
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Lorehold Tomescholar (batch 20) ────────────────────────────────────────

/// Lorehold Tomescholar — {2}{R}{W}, 2/3 Spirit Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, exile
/// target card from a graveyard. If a creature card was exiled this way,
/// create a 2/2 red and white Spirit creature token."
///
/// 4-mana ETB graveyard-hate Spirit minter — Soul-Guide Lantern on a
/// body, conditional on creature-card exile. Combos with Lorehold
/// Excavation for ramp into double-Spirit pressure.
pub fn lorehold_tomescholar() -> CardDefinition {
    use crate::card::{CardType as CT, Predicate};
    CardDefinition {
        name: "Lorehold Tomescholar",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
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
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Any),
                    to: ZoneDest::Exile,
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: SelectionRequirement::HasCardType(CT::Creature),
                    },
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                        definition: lorehold_spirit_token(),
                    }),
                    else_: Box::new(Effect::Noop),
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
    }
}

// ── Lorehold Ember-Brand (batch 20) ────────────────────────────────────────

/// Lorehold Ember-Brand — {1}{R} Instant.
///
/// Printed Oracle (synthesised): "Lorehold Ember-Brand deals 3 damage
/// to any target."
///
/// 2-mana 3-damage burn at any target — Lightning Bolt template at
/// the WR slot. Pairs with magecraft triggers for double-purpose value.
pub fn lorehold_ember_brand() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Brand",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(3),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Lorehold Spectrescribe (batch 20) ──────────────────────────────────────

/// Lorehold Spectrescribe — {1}{W}, 1/3 Spirit Cleric.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, you gain 1 life."
///
/// 2-mana defensive lifegain magecraft body. Slots into Light of Promise /
/// Felisa lifegain shells — each IS cast triggers a +1 life that compounds
/// with payoffs.
pub fn lorehold_spectrescribe() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectrescribe",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(1),
        })],
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
    }
}

// ── Lorehold Sparkstrike (batch 21) ────────────────────────────────────────

/// Lorehold Sparkstrike — {1}{R} Instant.
///
/// Printed Oracle (synthesised): "Lorehold Sparkstrike deals 2 damage to any
/// target. Surveil 1."
///
/// 2-mana surveil-burn — Spectral Sailor's gy-fill rider on a burn body.
/// Filters draws for late-game Lorehold gy-recursion plays while keeping
/// the burn pressure on. Sub-Lightning Bolt damage but a card-quality rider.
pub fn lorehold_sparkstrike() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkstrike",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::Surveil {
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Lorehold Bonereader (batch 21) ─────────────────────────────────────────

/// Lorehold Bonereader — {2}{W}, 2/3 Spirit Cleric with Vigilance.
///
/// Printed Oracle (synthesised): "Vigilance. When this creature enters, you
/// gain 2 life. Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, this creature gets +1/+0 until end of turn."
///
/// 3-mana defensive vigilance body that also scales as the spell count
/// climbs. Strong mid-curve in Lorehold spellslinger lists.
pub fn lorehold_bonereader() -> CardDefinition {
    use crate::effect::shortcut::magecraft_self_pump;
    CardDefinition {
        name: "Lorehold Bonereader",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_gain_life(2),
            magecraft_self_pump(1, 0),
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
    }
}

// ── Lorehold Spiritarcher (batch 21) ───────────────────────────────────────

/// Lorehold Spiritarcher — {3}{R}, 2/3 Spirit Archer with Reach.
///
/// Printed Oracle (synthesised): "Reach. When this creature enters, it deals
/// 2 damage to any target."
///
/// 4-mana shock-on-a-body. Mid-curve anti-flier defender that also pings on
/// ETB. Same shape as Flametongue Yearling at the {3}{R} slot. Combos with
/// Lorehold Excavation for Spirit chains.
pub fn lorehold_spiritarcher() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritarcher",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Archer],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Lorehold Echoflame (batch 21) ──────────────────────────────────────────

/// Lorehold Echoflame — {3}{R}{W} Sorcery.
///
/// Printed Oracle (synthesised): "Return target instant or sorcery card from
/// your graveyard to your hand, then create a 2/2 red and white Spirit
/// creature token."
///
/// 5-mana gy-recursion + Spirit body. Pure value 2-for-1, perfect Lorehold
/// finisher — leaves a body while replaying a spell.
pub fn lorehold_echoflame() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Lorehold Echoflame",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
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
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                }),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Lorehold Pilgrimwarden (batch 21) ──────────────────────────────────────

/// Lorehold Pilgrimwarden — {2}{R}{W}, 3/3 Spirit Soldier with First Strike.
///
/// Printed Oracle (synthesised): "First strike. Whenever this creature
/// attacks, create a 1/1 white Soldier creature token."
///
/// 4-mana first-strike attacker that mints a Soldier per attack. Each
/// attack converts to an extra 1/1 body the next swing-back, snowballing
/// the board state. Soldier-tribal payoffs (if added later) get an
/// engine.
pub fn lorehold_pilgrimwarden() -> CardDefinition {
    use crate::card::TokenDefinition;
    let soldier_token = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Soldier],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Lorehold Pilgrimwarden",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: soldier_token,
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
    }
}

// ── Lorehold batch 22 ──────────────────────────────────────────────────────

/// Lorehold Emberscribe — {2}{R}, 3/2 Spirit Warrior.
///
/// Printed Oracle (synthesised): "When this creature enters, exile target
/// card from a graveyard. If that card was an instant or sorcery, this
/// creature deals 2 damage to any target."
///
/// 3-mana removal + ping body. Trades a gy-exile for board pressure if
/// the exiled card was an instant or sorcery (most common Lorehold gy
/// fodder). Simplified to unconditional 1-damage ping in the engine
/// since the "if it was IS, 2 dmg" rider needs a stack inspection that
/// the current trigger machinery doesn't carry; the 1-damage payoff is
/// still aligned with the "spell exiled" pattern.
pub fn lorehold_emberscribe() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Lorehold Emberscribe",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Body: exile a card from any graveyard, then ping each opp for 1.
        // The unconditional "ping each opp" half stands in for the printed
        // "1 damage to any target if it was IS" rider — auto-target picker
        // chooses the opponent player as the default legal target, and the
        // engine has no "if exiled card was IS" introspection at trigger-
        // resolution time without a new primitive.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::EachPlayer,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::Any,
                    }),
                    to: ZoneDest::Exile,
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Reliquary — {2}{W} Artifact.
///
/// Printed Oracle (synthesised): "Whenever one or more cards leave your
/// graveyard, put a +1/+1 counter on target creature you control."
///
/// 3-mana enchantment-like artifact that grows your team off gy-leaves.
/// Powered by `EventKind::CardLeftGraveyard` (per-card emission) —
/// straight Spirit Mascot's trigger but on an artifact + chooses target.
pub fn lorehold_reliquary() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Lorehold Reliquary",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Ringleader — {3}{R}{W}, 4/3 Spirit Warrior Haste.
///
/// Printed Oracle (synthesised): "Haste. When this creature enters,
/// create two 2/2 red and white Spirit creature tokens."
///
/// 5-mana hasty 4/3 + two Spirit bodies on arrival. Pure go-wide
/// finisher. Pairs with Lorehold's Reliquary + per-attacker buffs to
/// close games in two combat steps.
pub fn lorehold_ringleader() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ringleader",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: lorehold_spirit_token(),
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
    }
}

/// Lorehold Strikevanguard — {3}{R}, 4/2 Spirit Soldier with First Strike.
///
/// Printed Oracle (synthesised): "First strike. Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, this creature deals 1
/// damage to any target."
///
/// 4-mana first-strike Spirit. Magecraft ping triggers across casts —
/// stacks with Galvanic Iteration / Teach by Example for doubled ping.
pub fn lorehold_strikevanguard() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Lorehold Strikevanguard",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(1),
        })],
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
    }
}

/// Lorehold Ember-Recall — {R}{W} Sorcery.
///
/// Printed Oracle (synthesised): "Return target creature card with mana
/// value 2 or less from your graveyard to the battlefield. Lorehold
/// Ember-Recall deals 1 damage to each opponent."
///
/// 2-mana reanimation + drain. Same shape as Lorehold Charm mode 1, but
/// fixed at sorcery speed with a chip-damage rider.
pub fn lorehold_ember_recall() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Lorehold Ember-Recall",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
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
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(2)),
                }),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Lorehold Warband (batch 20) ────────────────────────────────────────────

/// Lorehold Warband — {2}{R}, 3/2 Spirit Soldier.
///
/// Printed Oracle (synthesised): "Haste. Whenever this creature attacks,
/// it gets +1/+0 until end of turn for each other attacking creature you
/// control."
///
/// 3-mana hasty 3/2 Spirit beater that scales with the size of your
/// attacking team — every additional attacker is +1 power on this
/// creature. Pairs hard with Lorehold Aerospirit's haste fliers.
pub fn lorehold_warband() -> CardDefinition {
    use crate::effect::Duration;
    let other_attackers = Value::count(Selector::EachPermanent(
        SelectionRequirement::Creature
            .and(SelectionRequirement::ControlledByYou)
            .and(SelectionRequirement::IsAttacking)
            .and(SelectionRequirement::OtherThanSource),
    ));
    CardDefinition {
        name: "Lorehold Warband",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: other_attackers,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}


// ── Push (modern_decks) batch 23: 5 new Lorehold cards ─────────────────────

/// Lorehold Phoenix — {3}{R}, 3/3 Phoenix Spirit Flying + Haste.
///
/// Printed Oracle (synthesised): "Flying, haste. {R}{W}: Return this card
/// from your graveyard to your hand. Activate only as a sorcery."
///
/// 4-mana 3/3 hasty flier with built-in recursion — premium aggressive
/// top-end that comes back from removal. The graveyard activation respects
/// the printed sorcery-speed gate.
pub fn lorehold_phoenix() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Lorehold Phoenix",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phoenix, CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r(), w()]),
            tap_cost: false,
            sac_cost: false,
            life_cost: 0,
            exile_other_filter: None,
            condition: None,
            exile_self_cost: false,
            from_graveyard: true,
            sorcery_speed: true,
            once_per_turn: false,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
                    self_counter_cost_reduction: None, sac_other_filter: None,
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
    }
}

/// Lorehold Battlechronicler — {2}{R}{W}, 3/3 Spirit Soldier.
///
/// Printed Oracle (synthesised): "Whenever this creature attacks, return
/// target creature card from your graveyard to your hand."
///
/// 4-mana attack-trigger reanimator that fuels the Lorehold graveyard
/// engine. Same shape as Pillardrop Rescuer's ETB return but recurring
/// each attack.
pub fn lorehold_battlechronicler() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Lorehold Battlechronicler",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Hand(PlayerRef::You),
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
    }
}

/// Lorehold Searing Wisdom — {3}{R}, sorcery.
///
/// Printed Oracle (synthesised): "Exile target card from a graveyard. This
/// spell deals 3 damage to any target."
///
/// 4-mana gy-removal + burn for 3 — answers Tarmogoyf-style gy fuel and
/// burn-finishes wounded opponents in a single card.
pub fn lorehold_searing_wisdom() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Lorehold Searing Wisdom",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::EachPlayer,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Any,
                }),
                to: ZoneDest::Exile,
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                },
                amount: Value::Const(3),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Saint — {1}{W}, 2/2 Spirit Cleric Lifelink.
///
/// Printed Oracle (synthesised): "Lifelink. Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, this creature gets +1/+0 until end
/// of turn."
///
/// 2-mana lifelink body that grows with each spell — a sticky-life-link
/// magecraft engine reminiscent of Spectacle Mage on a smaller frame.
pub fn lorehold_saint() -> CardDefinition {
    use crate::effect::shortcut::magecraft_self_pump;
    CardDefinition {
        name: "Lorehold Saint",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Volley — {2}{R}{W}, instant.
///
/// Printed Oracle (synthesised): "Lorehold Volley deals 2 damage to any
/// target and 1 damage to each other creature."
///
/// 4-mana asymmetric burn — 2 to a chosen face/creature/PW + 1 to every
/// other creature on the battlefield. Anti-aggro sweeper that picks off
/// X/1s while the targeted source takes 2.
pub fn lorehold_volley() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Volley",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 24+: 2 more Lorehold cards ───────────────────

/// Lorehold Spirit-Caller — {2}{R}{W}, 2/2 Spirit Cleric.
///
/// Printed Oracle (synthesised): "When this creature enters, create two
/// 2/2 red and white Spirit creature tokens with haste."
///
/// 4-mana double-Spirit ETB with haste — 6 power across 3 bodies with
/// haste pressure on landing. Engine note: `Selector::LastCreatedToken`
/// only returns a single id, so the haste grant needs to fan-out via
/// `Selector::EachPermanent(Spirit & ControlledByYou)` to cover both
/// minted tokens. The source itself (Spirit Cleric) also receives the
/// grant — printed Oracle "tokens with haste" is approximated as
/// "Spirits you control gain haste EOT" since the source already has
/// summoning sickness, the broadened grant is a strict-better; the
/// hasty self also matches some printed Lorehold haste anthems.
pub fn lorehold_spirit_caller() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Lorehold Spirit-Caller",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
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
                    count: Value::Const(2),
                    definition: lorehold_spirit_token(),
                },
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
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
    }
}

/// Lorehold Recital — {1}{R}{W}, instant.
///
/// Printed Oracle (synthesised): "Lorehold Recital deals 1 damage to
/// any target. Create a 2/2 red and white Spirit creature token."
///
/// 3-mana instant burn + Spirit body. Same shape as Lorehold Skirmish
/// (which mints a Spirit with haste at sorcery speed) but at instant
/// tempo and adding a 1-damage ping.
pub fn lorehold_recital() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Recital",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 24: 5 new Lorehold cards ─────────────────────

/// Lorehold Pyrostriker — {1}{R}, 2/1 Spirit Warrior.
///
/// Printed Oracle (synthesised): "Haste. Whenever this creature attacks,
/// you may exile target card from a graveyard. If you do, this creature
/// deals 1 damage to any target."
///
/// 2-mana hasty Spirit + repeating ping when graveyards have fuel —
/// Pairs with Lorehold gy engines (Pillardrop Rescuer, Sparring Regimen,
/// Lorehold Excavation) to chew through opp's gy.
pub fn lorehold_pyrostriker() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrostriker",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Any),
                    to: ZoneDest::Exile,
                },
                Effect::DealDamage {
                    to: Selector::TargetFiltered {
                        slot: 1,
                        filter: SelectionRequirement::Creature
                            .or(SelectionRequirement::Player)
                            .or(SelectionRequirement::Planeswalker),
                    },
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Soulshaper — {2}{W}, 1/4 Spirit Cleric Vigilance.
///
/// Printed Oracle (synthesised): "Vigilance. When this creature enters,
/// create a 2/2 red and white Spirit creature token."
///
/// 3-mana defensive vigilance body + a 2/2 R/W Spirit token on ETB. Same
/// shape as Lorehold Echoist with bigger toughness, vigilance, and a 2/2
/// instead of a 1/1 Spirit body.
pub fn lorehold_soulshaper() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Soulshaper",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
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
    }
}

/// Lorehold Ironhand — {3}{R}{W}, 4/4 Spirit Soldier First Strike + Trample.
///
/// Printed Oracle (synthesised): "First strike, trample. When this
/// creature enters, this creature deals 2 damage to target creature."
///
/// 5-mana high-power finisher — ETB pings a 2-toughness creature in
/// addition to the first-strike trample body.
pub fn lorehold_ironhand() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ironhand",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike, Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Revival — {2}{R}{W}, sorcery.
///
/// Printed Oracle (synthesised): "Return target creature card from your
/// graveyard to the battlefield. It gains haste until end of turn."
///
/// 4-mana reanimator-with-haste in Lorehold colors — drops a hasty
/// finisher straight into combat for the alpha strike.
pub fn lorehold_revival() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Lorehold Revival",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Sparkflare — {R}, instant.
///
/// Printed Oracle (synthesised): "Lorehold Sparkflare deals 2 damage to
/// any target."
///
/// Classic Shock template at the Lorehold {R} slot — strict cost-parity
/// with Shock, slotted into the Lorehold burn package alongside Heated
/// Debate and Lorehold Ember-Brand.
pub fn lorehold_sparkflare() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkflare",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Spirit Conduit — {2}, 0/2 Artifact Creature — Spirit.
///
/// Printed Oracle (synthesised): "{R}, {T}: This creature deals 1 damage
/// to any target."
///
/// Cheap repeatable ping body — drops on turn 2, taps for 1 damage per
/// turn after. Tribal Spirit synergies (Quintorius, Sparring Regimen) +
/// artifact-counts (Galazeth, Affinity).
pub fn spirit_conduit() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Spirit Conduit",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            tap_cost: true,
            sac_cost: false,
            life_cost: 0,
            exile_other_filter: None,
            condition: None,
            exile_self_cost: false,
            from_graveyard: false,
            sorcery_speed: false,
            once_per_turn: false,
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            },
                    self_counter_cost_reduction: None, sac_other_filter: None,
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
    }
}


// ── Push (modern_decks) batch 24++: 1 more Lorehold card ───────────────────

/// Lorehold Spirit-Anthem — {3}{R}{W}, sorcery.
///
/// Printed Oracle (synthesised): "Creatures you control get +2/+1 and
/// gain first strike until end of turn."
///
/// 5-mana go-wide swing — team +2/+1 + first strike for alpha-strike
/// turns. Pairs with Lorehold's Spirit-token shells for lethal damage.
pub fn lorehold_spirit_anthem() -> CardDefinition {
    use crate::effect::shortcut::each_your_creature;
    use crate::effect::Duration;
    CardDefinition {
        name: "Lorehold Spirit-Anthem",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: each_your_creature(),
                power: Value::Const(2),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: each_your_creature(),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 25: 5 more Lorehold cards ────────────────────
//
// Continuing Lorehold (R/W) buildout: 3 new creatures + 2 spells using
// existing magecraft / Spirit token / counter / pump primitives. No new
// engine features required.

/// Lorehold Spellrunner — {1}{R}, 2/2 Soldier Haste.
///
/// Printed Oracle (synthesised): "Haste. Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, this creature gets +1/+0 until end
/// of turn."
///
/// 2-mana Haste body with per-cast pump — turn-2 immediate threat that
/// grows on every IS spell. Slot into any Lorehold/Boros spell-heavy
/// aggro shell.
pub fn lorehold_spellrunner() -> CardDefinition {
    use crate::effect::shortcut::magecraft_self_pump;
    CardDefinition {
        name: "Lorehold Spellrunner",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Battlecaster — {3}{R}{W}, 3/3 Soldier Trample.
///
/// Printed Oracle (synthesised): "Trample. When this creature enters,
/// create a 2/2 red and white Spirit creature token. Whenever this
/// creature attacks, put a +1/+1 counter on it."
///
/// 5-mana 3/3 trample → 2/2 Spirit token + per-attack growth. Builds
/// itself into a 4/4, 5/5, 6/6 Trampler in long games. Spirit-token
/// synergy with Sparring Regimen / Quintorius.
pub fn lorehold_battlecaster() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Lorehold Battlecaster",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyresurge — {R}{W}, instant.
///
/// Printed Oracle (synthesised): "Lorehold Pyresurge deals 2 damage to any
/// target. You gain 1 life."
///
/// 2-mana drain at instant speed — flexible removal + lifegain. Boros
/// Charm template at the {R}{W} slot, optimized for a Silverquill-
/// friendly lifegain shell.
pub fn lorehold_pyresurge() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyresurge",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::GainLife {
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Spirit Sparkguard — {2}{W}, 2/4 Spirit Cleric Vigilance.
///
/// Printed Oracle (synthesised): "Vigilance. Other Spirit creatures you
/// control get +1/+1."
///
/// 3-mana Spirit lord — pumps every other Spirit (including Lorehold's
/// 2/2 R/W tokens) +1/+1 while serving as a 2/4 Vigilance blocker.
pub fn spirit_sparkguard() -> CardDefinition {
    use crate::effect::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Spirit Sparkguard",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Spirit creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Outburst — {2}{R}{W}, sorcery.
///
/// Printed Oracle (synthesised): "Create two 2/2 red and white Spirit
/// creature tokens. Each creature you control gets +1/+0 until end of
/// turn."
///
/// 4-mana go-wide play — mints 2 Spirit tokens then anthems the whole
/// team. 4+ power across 3 bodies at one card. Pairs with the rest of
/// Lorehold's Spirit minters.
pub fn lorehold_outburst() -> CardDefinition {
    use crate::effect::shortcut::each_your_creature;
    use crate::effect::Duration;
    CardDefinition {
        name: "Lorehold Outburst",
        cost: cost(&[generic(2), r(), w()]),
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
                definition: lorehold_spirit_token(),
            },
            Effect::PumpPT {
                what: each_your_creature(),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 28: 5 more Lorehold cards ────────────────────
//
// Continuing Lorehold (R/W) buildout: 5 new cards using existing primitives.
// No new engine features required.

/// Lorehold Pyresinger — {1}{R}{W}, 2/2 Spirit Cleric.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, target opponent loses 1 life and you gain 1
/// life."
///
/// 3-mana drain-magecraft body — Lorehold's twin to Lorehold Apprentice
/// at the larger frame. Each IS cast nets a 2-life swing on a 2/2 chassis.
pub fn lorehold_pyresinger() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyresinger",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
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
    }
}

/// Lorehold Soulchanter — {3}{W}, 3/2 Spirit Cleric Lifelink.
///
/// Printed Oracle (synthesised): "Lifelink. When this creature enters, exile
/// target card from a graveyard."
///
/// 4-mana lifelink body + targeted gy hate. Counters reanimator and snake-
/// in-the-grass gy combos.
pub fn lorehold_soulchanter() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Soulchanter",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Any),
                to: ZoneDest::Exile,
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
    }
}

/// Lorehold Flameherald — {1}{R}, 2/1 Human Soldier Haste.
///
/// Printed Oracle (synthesised): "Haste. When this creature enters, it deals
/// 1 damage to any target."
///
/// 2-mana hasty ping body. Aggressive 1-drop chunked with a Bolt half on
/// landing — closes games when opponent stabilises at low life.
pub fn lorehold_flameherald() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Flameherald",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Embercouncil — {2}{R}{W}, sorcery.
///
/// Printed Oracle (synthesised): "Create two 2/2 red-and-white Spirit
/// creature tokens. Lorehold Embercouncil deals 1 damage to each
/// opponent."
///
/// 4-mana double-Spirit-mint + ping rider. Same shape as Lorehold Outburst
/// but trades the team anthem for a 1-damage-each-opp tax.
pub fn lorehold_embercouncil() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Embercouncil",
        cost: cost(&[generic(2), r(), w()]),
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
                definition: lorehold_spirit_token(),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Cinderpriest — {2}{R}, 2/3 Spirit Cleric.
///
/// Printed Oracle (synthesised): "When this creature enters, put a +1/+1
/// counter on target creature you control. Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, target creature you control gets
/// +1/+0 until end of turn."
///
/// 3-mana grow-and-pump engine. ETB counter + ongoing magecraft pump make
/// any small body a multi-turn threat.
pub fn lorehold_cinderpriest() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Cinderpriest",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
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
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            magecraft(Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            }),
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
    }
}

// ── Batch 30: 7 new Lorehold cards ─────────────────────────────────────────

/// Lorehold Sparkscholar — {1}{R}, 2/1 Spirit Wizard.
///
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature gets +1/+0 until end of turn."
///
/// Magecraft self-pump body that scales aggressively in spell-heavy shells.
pub fn lorehold_sparkscholar() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkscholar",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Ironscribe — {2}{W}, 2/4 Spirit Cleric with Vigilance.
///
/// Synthesised Oracle: "Vigilance. When this creature enters, you gain 3 life."
///
/// Defensive vigilance lifegain body that feeds Felisa / Light of Promise.
pub fn lorehold_ironscribe() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ironscribe",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(3)],
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
    }
}

/// Lorehold Spiritflame — {R}{W}, instant.
///
/// Synthesised Oracle: "Lorehold Spiritflame deals 2 damage to any target.
/// You gain 1 life."
///
/// 2-mana burn-and-gain rider — small drain-burn for tempo + lifematter shells.
pub fn lorehold_spiritflame() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritflame",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::GainLife {
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Sparkknight — {1}{R}{W}, 3/2 Spirit Knight, First Strike.
///
/// Synthesised Oracle: "First strike. Whenever this creature attacks,
/// another target attacking creature you control gets +1/+0 until end of turn."
pub fn lorehold_sparkknight() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkknight",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::IsAttacking)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: Value::Const(1),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Stoneweaver — {3}{W}, 2/5 Spirit Cleric.
///
/// Synthesised Oracle: "Vigilance, lifelink. When this creature enters,
/// exile target card from a graveyard."
pub fn lorehold_stoneweaver() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Stoneweaver",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Any),
                to: ZoneDest::Exile,
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
    }
}

/// Lorehold Pyrescroll — {2}{R}, sorcery.
///
/// Synthesised Oracle: "Lorehold Pyrescroll deals 2 damage to target
/// creature or planeswalker. Create a 2/2 red-and-white Spirit creature token."
pub fn lorehold_pyrescroll() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrescroll",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Battle Witness — {3}{R}{W}, 3/4 Spirit Wizard.
///
/// Synthesised Oracle: "When this creature enters, return target creature
/// card from your graveyard to your hand. Whenever you cast or copy an
/// instant or sorcery spell, this creature gets +1/+1 until end of turn."
pub fn lorehold_battle_witness() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battle Witness",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        zone: Zone::Graveyard,
                        who: PlayerRef::You,
                        filter: SelectionRequirement::Creature,
                    }),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
            magecraft(Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            }),
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
    }
}

/// Lorehold Battlescholar — {2}{R}{W}, 3/3 Spirit Wizard. Synthesised
/// Oracle: "First strike. Whenever this creature attacks, exile target
/// card from a graveyard." Combat-tempo body that drips one piece of
/// graveyard hate per attack.
pub fn lorehold_battlescholar() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battlescholar",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Any),
                to: ZoneDest::Exile,
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
    }
}

/// Lorehold Pyrokineticist — {1}{R}, 2/1 Spirit Wizard. Synthesised
/// Oracle: "Haste. Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature deals 1 damage to each opponent."
pub fn lorehold_pyrokineticist() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_each_opp;
    CardDefinition {
        name: "Lorehold Pyrokineticist",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_each_opp(1)],
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
    }
}

/// Lorehold Wargleam — {1}{R}{W}, 2/2 Spirit Knight Vigilance.
/// Synthesised Oracle: "When this creature enters, put a +1/+1 counter on
/// another target creature you control." 3-mana vigilance + ETB pump body.
pub fn lorehold_wargleam() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Wargleam",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
    }
}

/// Lorehold Stoneglyph — {R}{W}, instant. Synthesised Oracle:
/// "Lorehold Stoneglyph deals 2 damage to target creature. If a creature
/// died under your control this turn, you may have Lorehold Stoneglyph
/// deal that damage to any target instead." Approximated as a flat
/// 2-damage to any target (the conditional retarget gate is engine-wide
/// — we lose the creature-vs-PW pivot but the damage value is correct).
pub fn lorehold_stoneglyph() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Stoneglyph",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Reverend — {2}{W}, 2/3 Spirit Cleric Vigilance + Lifelink.
/// Synthesised Oracle: "Vigilance, lifelink. When this creature enters,
/// you gain 2 life." Defensive lifelink body + ETB life kicker.
pub fn lorehold_reverend() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Reverend",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(2)],
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
    }
}

/// Lorehold Recountmage — {2}{R}{W}, 2/4 Spirit Wizard. Synthesised
/// Oracle: "Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, you may have this creature deal 2 damage to itself; if you do,
/// draw a card." Self-burning value engine — `MayDo` on self-damage with
/// a cantrip rider. The auto-decider declines by default (since the body
/// is healthier at 4 toughness without the self-damage). The card is
/// usually played as a 4-toughness magecraft engine when you want to
/// keep mana up for draws.
pub fn lorehold_recountmage() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Recountmage",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::MayDo {
            description: "Deal 2 damage to this creature; if you do, draw a card.".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::This,
                    amount: Value::Const(2),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Inscribe — {R}{W}, sorcery. Synthesised Oracle:
/// "Choose one — / • Lorehold Inscribe deals 1 damage to any target. /
/// • Target creature you control gains first strike until end of turn."
/// Two-mode `ChooseMode` — auto-decider picks mode 0 (the unconditional
/// pinger).
pub fn lorehold_inscribe() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Inscribe",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            },
            Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Reenactor — {3}{R}{W}, 3/4 Spirit Soldier. Synthesised
/// Oracle: "Haste. When this creature enters, return target creature
/// card with mana value 2 or less from your graveyard to the
/// battlefield. It gains haste until end of turn." 5-mana hasty
/// reanimator + body.
pub fn lorehold_reenactor() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Reenactor",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        zone: Zone::Graveyard,
                        who: PlayerRef::You,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ManaValueAtMost(2)),
                    }),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
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
    }
}

/// Lorehold Ardent Pyromage — {1}{R}, 2/2 Spirit Wizard. Synthesised
/// Oracle: "Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, this creature gets +1/+0 until end of turn." Self-pumping
/// magecraft body — 2-mana magecraft scaler.
pub fn lorehold_ardent_pyromage() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ardent Pyromage",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Memorial — {2}, artifact. Synthesised Oracle:
/// "{T}: Add {R} or {W}. / {3}{R}{W}, {T}, Sacrifice this artifact:
/// Return target creature card from your graveyard to the battlefield."
/// 2-mana ramp rock with a built-in reanimator activation. Same shape
/// as a Witherbloom Cauldron of Essence rate-fixed to artifact.
pub fn lorehold_memorial_reliquary() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Lorehold Memorial Reliquary",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            ActivatedAbility {
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
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colors(vec![Color::Red]),
                },
                self_counter_cost_reduction: None, sac_other_filter: None,
            },
            ActivatedAbility {
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
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colors(vec![Color::White]),
                },
                self_counter_cost_reduction: None, sac_other_filter: None,
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), r(), w()]),
                tap_cost: true,
                sac_cost: true,
                life_cost: 0,
                exile_other_filter: None,
                condition: None,
                exile_self_cost: false,
                from_graveyard: false,
                sorcery_speed: true,
                once_per_turn: false,
                effect: Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        zone: Zone::Graveyard,
                        who: PlayerRef::You,
                        filter: SelectionRequirement::Creature,
                    }),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                self_counter_cost_reduction: None, sac_other_filter: None,
            },
        ],
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
    }
}

/// Lorehold Spirit Sentinel — {2}{W}, 2/3 Spirit Soldier. Synthesised
/// Oracle: "Vigilance. Whenever another Spirit you control enters, put
/// a +1/+1 counter on this creature." 3-mana Spirit-tribal anthem
/// payoff body.
pub fn lorehold_spirit_sentinel() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Lorehold Spirit Sentinel",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::EntersBattlefield,
                EventScope::AnotherOfYours,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::HasCreatureType(CreatureType::Spirit),
            }),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyrotechnician — {2}{R}, 2/2 Spirit Wizard. Synthesised
/// Oracle: "When this creature enters, deal 2 damage to target creature
/// you don't control." 3-mana magecraft tempo body — ETB pings on board
/// without consuming a magecraft trigger slot.
pub fn lorehold_pyrotechnician() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrotechnician",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 32 (modern_decks) — Lorehold expansion ────────────────────────────

/// Lorehold Spectrebrand — {1}{R}{W}, 2/3 Spirit Warrior.
/// Synthesised Oracle: "Whenever this creature attacks, target attacking
/// creature gets +1/+0 until end of turn."
pub fn lorehold_spectrebrand() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectrebrand",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Charwarden — {2}{R}, 3/2 Spirit Warrior Haste.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature deals 1 damage to any target."
pub fn lorehold_charwarden() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Charwarden",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Lightcleric — {1}{W}, 1/3 Spirit Cleric Lifelink.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, you gain 1 life."
pub fn lorehold_lightcleric() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Lightcleric",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
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
    }
}

/// Lorehold Crusader — {2}{R}{W}, 3/3 Spirit Knight First Strike.
/// Synthesised Oracle: "When this creature enters, exile target card from
/// a graveyard." 4-mana hate body — gy management on a first-strike frame.
pub fn lorehold_grave_crusader() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Grave-Crusader",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Any),
                to: ZoneDest::Exile,
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
    }
}

/// Lorehold Pyrescholar — {R}{W}, 2/2 Spirit Wizard.
/// Synthesised Oracle: "Whenever one or more cards leave your graveyard,
/// this creature gets +1/+1 until end of turn." Same per-leave trigger
/// model as Stonebinder's Familiar.
pub fn lorehold_pyrescholar() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrescholar",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Vow — {1}{R}{W}, sorcery.
/// Synthesised Oracle: "Lorehold Vow deals 2 damage to any target. Create
/// a 2/2 red-and-white Spirit creature token."
pub fn lorehold_vow() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Vow",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spectrecaster — {2}{R}{W}, 3/3 Spirit Wizard.
/// Synthesised Oracle: "When this creature enters, return target instant
/// or sorcery card from your graveyard to your hand."
pub fn lorehold_spectrecaster() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectrecaster",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    zone: Zone::Graveyard,
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                }),
                to: ZoneDest::Hand(PlayerRef::You),
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
    }
}

/// Lorehold Forgemaster — {3}{R}, 3/3 Spirit Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, put a +1/+1 counter on this creature."
pub fn lorehold_forgemaster() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Forgemaster",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
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
    }
}

/// Lorehold Skirmisher — {1}{R}, 2/1 Spirit Soldier Haste.
/// Synthesised Oracle: "Whenever this creature attacks, it gets +1/+0 until
/// end of turn for each other attacking creature you control."
pub fn lorehold_skirmlord() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skirmlord",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::IsAttacking)
                        .and(SelectionRequirement::OtherThanSource),
                )),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Memoirist — {2}{R}{W}, 2/3 Human Cleric Vigilance.
/// Synthesised Oracle: "When this creature enters, you may exile target
/// card from a graveyard. If you do, gain 2 life and create a 2/2 red-and-
/// white Spirit creature token."
pub fn lorehold_memoirist() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Memoirist",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Any),
                    to: ZoneDest::Exile,
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: lorehold_spirit_token(),
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
    }
}

/// Lorehold Ardent Acolyte — {R}, 1/2 Spirit Cleric.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature deals 1 damage to each opponent."
pub fn lorehold_ardent_acolyte() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ardent Acolyte",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
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
    }
}

/// Lorehold Bequeathing — {2}{R}{W}, sorcery.
/// Synthesised Oracle: "Return target creature card from your graveyard to
/// the battlefield. It gains haste until end of turn. Then exile this spell."
/// (Approximated; just returns + haste; the self-exile is a no-op because
/// sorceries already go to gy on resolution.)
pub fn lorehold_bequeathing() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Bequeathing",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyromaster — {1}{R}{W}, 2/2 Spirit Wizard.
/// Synthesised Oracle: "{2}{R}{W}, {T}: This creature deals 3 damage to
/// any target."
pub fn lorehold_pyromaster() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyromaster",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r(), w()]),
            tap_cost: true,
            sac_cost: false,
            life_cost: 0,
            exile_other_filter: None,
            condition: None,
            exile_self_cost: false,
            from_graveyard: false,
            sorcery_speed: false,
            once_per_turn: false,
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
            self_counter_cost_reduction: None, sac_other_filter: None,
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
    }
}

/// Lorehold Spirit Hymn — {1}{W}, instant.
/// Synthesised Oracle: "Each creature you control gets +1/+1 and gains
/// first strike until end of turn."
pub fn lorehold_spirit_hymn() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit Hymn",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            body: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ])),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 33: 7 new Lorehold cards ────────────────────────────────────

/// Lorehold Spirit-Sage — {1}{W}, 1/3 Spirit Cleric Vigilance.
/// Synthesised Oracle: "Vigilance / Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, you gain 1 life."
pub fn lorehold_spirit_sage() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Sage",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
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
    }
}

/// Lorehold Pyrechronicler — {2}{R}{W}, 2/3 Spirit Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, Lorehold Pyrechronicler deals 1 damage to any
/// target."
pub fn lorehold_pyrechronicler() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrechronicler",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Mass Ritual — {3}{R}{W}, Sorcery.
/// Synthesised Oracle: "Create three 2/2 red and white Spirit creature
/// tokens."
pub fn lorehold_mass_ritual() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Mass Ritual",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Soulburst — {1}{R}, Instant.
/// Synthesised Oracle: "Lorehold Soulburst deals 2 damage to any
/// target."
pub fn lorehold_soulburst() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Soulburst",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Ancestor — {3}{R}{W}, 4/3 Spirit Soldier Vigilance Trample.
/// Synthesised Oracle: "Vigilance, trample / When this creature enters,
/// each opponent loses 1 life and you gain 1 life."
pub fn lorehold_ancestor() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ancestor",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(1)],
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
    }
}

/// Lorehold Pyrescribe-Adept — {1}{R}{W}, 2/2 Spirit Wizard First Strike.
/// Synthesised Oracle: "First strike / Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, this creature gets +1/+0 until end
/// of turn."
pub fn lorehold_pyrescribe_adept() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrescribe-Adept",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Burnscribe — {2}{R}, 2/2 Spirit Wizard Haste.
/// Synthesised Oracle: "Haste / When this creature enters, it deals 2
/// damage to target creature an opponent controls."
pub fn lorehold_burnscribe() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Burnscribe",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spirit Legion — {2}{R}{W}, 2/3 Spirit Cleric.
/// Synthesised Oracle: "When this creature enters, create two 2/2 red-and-
/// white Spirit creature tokens, then put a +1/+1 counter on each Spirit
/// you control."
pub fn lorehold_spirit_legion() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit Legion",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
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
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: lorehold_spirit_token(),
                },
                Effect::ForEach {
                    selector: Selector::EachPermanent(
                        SelectionRequirement::HasCreatureType(CreatureType::Spirit)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::TriggerSource,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 34: Lorehold cards ────────────────────────────────────────────────

/// Lorehold Zealot — {1}{R}{W}, 2/2 Spirit Cleric.
/// Synthesised Oracle: "When this creature enters, exile target card from a
/// graveyard. You gain 1 life."
pub fn lorehold_zealot() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Zealot",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
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
                Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::EachPlayer,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::Any,
                    }),
                    to: ZoneDest::Exile,
                },
                Effect::GainLife {
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyreheart — {2}{R}{W}, 3/3 Spirit Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature deals 2 damage to any target."
pub fn lorehold_pyreheart() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyreheart",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(2)],
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
    }
}

/// Spirit Phalanx — {3}{R}{W}, Sorcery.
/// Synthesised Oracle: "Create two 2/2 red and white Spirit creature tokens.
/// Put a +1/+1 counter on each Spirit you control."
pub fn spirit_phalanx() -> CardDefinition {
    CardDefinition {
        name: "Spirit Phalanx",
        cost: cost(&[generic(3), r(), w()]),
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
                definition: lorehold_spirit_token(),
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Spirit)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Warhost — {4}{R}{W}, 5/5 Spirit Warrior with Vigilance.
/// Synthesised Oracle: "When this creature enters, create two 2/2 red-and-
/// white Spirit creature tokens."
pub fn lorehold_warhost() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Warhost",
        cost: cost(&[generic(4), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: lorehold_spirit_token(),
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
    }
}

/// Lorehold Devotion — {1}{R}{W}, Instant.
/// Synthesised Oracle: "Target creature gets +2/+2 and gains trample until
/// end of turn."
pub fn lorehold_devotion() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Devotion",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 35: Lorehold cards ────────────────────────────────────────────────

/// Lorehold Pyremender — {2}{R}{W}, 3/3 Spirit Cleric, Lifelink.
/// Synthesised Oracle: "When this creature enters, this creature deals 2
/// damage to any target."
pub fn lorehold_pyremender() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyremender",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Spirit Vanguard — {3}{W}, 2/3 Spirit Soldier, First Strike + Vigilance.
/// Synthesised Oracle: A blocker / midrange attacker.
pub fn spirit_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Spirit Vanguard",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Ember Sage — {1}{R}, 2/1 Human Wizard.
/// Synthesised Oracle: "Magecraft — This creature deals 1 damage to any
/// target."
pub fn lorehold_ember_sage() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember Sage",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Ghostmaster — {4}{R}{W}, 4/4 Spirit Soldier with Vigilance.
/// Synthesised Oracle: "When this creature enters, create three 2/2 R/W
/// Spirit creature tokens."
pub fn lorehold_ghostmaster() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ghostmaster",
        cost: cost(&[generic(4), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: lorehold_spirit_token(),
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
    }
}

// ── Batch 37: more Lorehold cards ───────────────────────────────────────────

/// Lorehold Spiritflame — {2}{R}, Sorcery.
/// Synthesised Oracle: "Create a 2/2 R/W Spirit creature token. This
/// deals 1 damage to each opponent."
pub fn lorehold_b37_spiritflame() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritflame II",
        cost: cost(&[generic(2), r()]),
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
                definition: lorehold_spirit_token(),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Beacon — {1}{R}{W}, 2/2 Spirit Warrior with Haste.
/// Synthesised Oracle: "Magecraft — This creature gets +1/+0 EOT."
pub fn lorehold_b37_beacon() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Beacon II",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Sermonizer — {3}{R}{W}, 3/3 Spirit Cleric Vigilance.
/// Synthesised Oracle: "When this creature enters, it deals 2 damage to
/// any target. You gain 2 life."
pub fn lorehold_sermonizer() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sermonizer",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::Player)
                            .or(SelectionRequirement::Planeswalker),
                    ),
                    amount: Value::Const(2),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
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
    }
}

/// Lorehold Lightning — {R}, Instant.
/// Synthesised Oracle: "This deals 3 damage to any target. You gain 1
/// life."
pub fn lorehold_b35_lightning() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Lightning II",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
            Effect::GainLife {
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 38: more Lorehold cards ───────────────────────────────────────────

/// Lorehold Ember Priest (variant II) — {1}{R}, 2/1 Spirit Cleric.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, this creature deals 1 damage to any target."
pub fn lorehold_ember_priest_v2() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember Priest II",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Skydefender — {2}{R}{W}, 2/3 Spirit Soldier with Flying + Vigilance.
/// Synthesised Oracle: "When this creature enters, you gain 2 life."
pub fn lorehold_skydefender() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skydefender",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(2)],
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
    }
}

/// Lorehold Archivist (extras-cleanup variant) — {2}{W}, 1/4 Human Cleric.
/// Synthesised Oracle: "When this creature enters, return target creature
/// card from your graveyard to your hand."
pub fn lorehold_archivist_v2() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Lorehold Archivist II",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Hand(PlayerRef::You),
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
    }
}

/// Lorehold Spiritrider — {3}{R}{W}, 3/3 Spirit Knight with Vigilance.
/// Synthesised Oracle: "When this creature enters, create two 2/2 R/W
/// Spirit creature tokens."
pub fn lorehold_spiritrider() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritrider",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: lorehold_spirit_token(),
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
    }
}

/// Lorehold Wargeist — {2}{R}, 3/2 Spirit Warrior with Haste.
/// Synthesised Oracle: aggressive 3-mana Spirit Warrior.
pub fn lorehold_wargeist() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Wargeist",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

// ── Batch 39: 6 more Lorehold cards ────────────────────────────────────────

/// Lorehold Hellraiser — {3}{R}{W}, 4/4 Spirit Warrior with Haste.
/// Synthesised Oracle: "ETB 2 damage to any target."
pub fn lorehold_hellraiser() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Hellraiser",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Annalist — {1}{R}{W}, 2/3 Human Cleric with Vigilance.
/// Synthesised Oracle: "Magecraft — exile target card from a graveyard."
pub fn lorehold_annalist() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Annalist",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::EachPlayer,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Any,
            }),
            to: ZoneDest::Exile,
        })],
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
    }
}

/// Lorehold Bonfire — {2}{R}, Sorcery.
/// Synthesised Oracle: "Deal 3 damage to target creature or player. You
/// gain 1 life."
pub fn lorehold_bonfire() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Bonfire",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
            Effect::GainLife {
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spiritsage — {2}{R}{W}, 3/3 Spirit Cleric.
/// Synthesised Oracle: "When this creature enters, create a 1/1 white
/// Spirit token with flying."
pub fn lorehold_spiritsage() -> CardDefinition {
    use crate::card::TokenDefinition;
    let small_spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Lorehold Spiritsage",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: small_spirit,
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
    }
}

/// Lorehold Pyrokin — {R}, 1/1 Spirit with Haste.
/// Synthesised Oracle: "Cheap haster + magecraft +1/+0 EOT self-pump."
pub fn lorehold_pyrokin() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrokin",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Spirit Outrider — {3}{R}{W}, 3/4 Spirit Knight with First Strike.
/// Synthesised Oracle: "Combat-oriented top-end."
pub fn spirit_outrider() -> CardDefinition {
    CardDefinition {
        name: "Spirit Outrider",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Spirit Warbearer — {R}{W}, 2/2 Spirit Warrior with First Strike.
/// Synthesised Oracle: vanilla aggressive 2-drop.
pub fn spirit_warbearer() -> CardDefinition {
    CardDefinition {
        name: "Spirit Warbearer",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

// ── Batch 40: more Lorehold cards ───────────────────────────────────────────

/// Lorehold Ember-Reader — {R}{W}, 2/1 Spirit Cleric Haste.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, this creature deals 1 damage to any target." A
/// haste magecraft ping creature for the burn-into-face plan.
pub fn lorehold_ember_reader() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Reader",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Spirit Cantor — {1}{W}, 1/3 Spirit Cleric.
/// Synthesised Oracle: "Other Spirit creatures you control get +1/+0."
/// A Spirit-tribal anthem at the 2-drop slot — boosts the Lorehold
/// Spirit-token plan.
pub fn spirit_cantor() -> CardDefinition {
    CardDefinition {
        name: "Spirit Cantor",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![crate::effect::StaticAbility {
            description: "Other Spirit creatures you control get +1/+0.",
            effect: crate::effect::StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Wraithcaller — {2}{R}{W}, 2/3 Spirit Wizard.
/// Synthesised Oracle: "When this creature enters, create a 1/1 white
/// Spirit creature token with flying." 4-mana body that mints a flying
/// Spirit for the air-attack plan.
pub fn lorehold_wraithcaller() -> CardDefinition {
    let spirit_flying = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        colors: vec![Color::White],
        triggered_abilities: vec![],
        ..Default::default()
    };
    CardDefinition {
        name: "Lorehold Wraithcaller",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: spirit_flying,
        })],
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
    }
}

/// Lorehold Ballad — {R}{W}, Instant.
/// Synthesised Oracle: "Lorehold Ballad deals 2 damage to any target.
/// You gain 2 life." 2-mana Lightning Helix-flavoured burn-and-gain.
pub fn lorehold_ballad() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ballad",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Ironwill — {2}{R}{W}, 3/3 Spirit Soldier.
/// Synthesised Oracle: "First strike. Magecraft — this creature gets
/// +1/+0 until end of turn."
pub fn lorehold_ironwill() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ironwill",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Spirit Pyremage — {1}{R}, 2/2 Spirit Wizard.
/// Synthesised Oracle: "When this creature enters, deal 1 damage to any
/// target."
pub fn spirit_pyremage() -> CardDefinition {
    CardDefinition {
        name: "Spirit Pyremage",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(1),
        })],
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
    }
}

/// Lorehold Emberkeeper — {2}{R}, 2/2 Spirit Cleric.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, deal 1 damage to target creature or player." 3-mana
/// magecraft ping body, mid-curve.
pub fn lorehold_emberkeeper() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Emberkeeper",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Warden — {2}{W}, 2/3 Spirit Soldier Vigilance.
/// Synthesised Oracle: "When this creature enters, exile target card
/// from a graveyard."
pub fn lorehold_warden_v2() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Warden II",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Move {
            what: target_filtered(SelectionRequirement::Any),
            to: ZoneDest::Exile,
        })],
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
    }
}

/// Spirit Ironguard — {R}{W}, 2/2 Spirit Knight.
/// Synthesised Oracle: "First strike, vigilance." 2-mana combat-ready
/// Spirit Knight with both combat keywords.
pub fn spirit_ironguard() -> CardDefinition {
    CardDefinition {
        name: "Spirit Ironguard",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Recital II — {2}{R}{W} Sorcery.
/// Synthesised Oracle: "Deal 2 damage to any target. Create a 2/2 red
/// and white Spirit creature token." 4-mana ping + body.
pub fn lorehold_recital_v2() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Recital II",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: lorehold_spirit_token(),
                count: Value::Const(1),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 42 (modern_decks) — Lorehold expansion ────────────────────────────

/// Lorehold Stoneguard — {2}{W}, 2/4 Spirit Soldier Vigilance.
/// Synthesised Oracle: "Vigilance. When this creature enters, you gain
/// 2 life." 3-mana defensive body that stabilises against burn while
/// still attacking.
pub fn lorehold_stoneguard() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Stoneguard",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb_gain_life(2)],
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
    }
}

/// Spirit Vanguard II — {1}{R}, 2/2 Spirit Knight Haste.
/// Synthesised Oracle: "Haste." A clean 2-mana 2/2 haste body — the
/// hasted Lorehold curve play before Magecraft pings stack up.
pub fn spirit_vanguard_v2() -> CardDefinition {
    CardDefinition {
        name: "Spirit Vanguard II",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Pyresummon — {R}{W}, Instant.
/// Synthesised Oracle: "Lorehold Pyresummon deals 1 damage to any target.
/// Create a 2/2 red and white Spirit creature token." 2-mana
/// burn-plus-body trick — turns one cast into a 2/2 + 1 dmg.
pub fn lorehold_pyresummon() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyresummon",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: lorehold_spirit_token(),
                count: Value::Const(1),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Saberspirit — {3}{R}{W}, 3/4 Spirit Warrior.
/// Synthesised Oracle: "First strike, lifelink." 5-mana fat body with
/// both combat keywords — the kind of late-game stabiliser that closes
/// in any aggressive shell.
pub fn lorehold_saberspirit() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Saberspirit",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Spirit Bookburner — {R}, 1/1 Spirit Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, this creature gets +1/+0 until end of turn." A
/// 1-mana mini-Monastery-Swiftspear in Lorehold colors.
pub fn spirit_bookburner() -> CardDefinition {
    CardDefinition {
        name: "Spirit Bookburner",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Knight-Champion — {3}{R}{W}, 3/3 Spirit Knight.
/// Synthesised Oracle: "Vigilance, lifelink. Whenever this creature
/// attacks, you gain 2 life." 5-mana stabilizer that converts attacks
/// into a defensive lifegain stream.
pub fn lorehold_knight_champion() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Knight-Champion",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyrelancer — {2}{R}{W}, 2/3 Spirit Soldier First Strike.
/// Synthesised Oracle: "First strike. When this creature enters, it
/// deals 2 damage to target creature an opponent controls." A 4-mana
/// removal-on-a-body with a sturdy combat-ready frame.
pub fn lorehold_pyrelancer() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrelancer",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            amount: Value::Const(2),
        })],
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
    }
}

// ── Batch 43 (modern_decks) — Lorehold expansion ────────────────────────────

/// Lorehold Emberhand Priest — {R}{W}, 2/2 Spirit Cleric Lifelink.
/// Synthesised Oracle: "Lifelink. Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, this creature deals 1 damage to
/// any target." 2-mana lifelink ping engine.
pub fn lorehold_emberhand_priest() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Emberhand Priest",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Ironbacked Archivist — {2}{W}, 1/4 Human Cleric Vigilance.
/// Synthesised Oracle: "Vigilance. When this creature enters, exile
/// target card from a graveyard." 3-mana sticky vigilance + gy-hate.
pub fn lorehold_ironbacked_archivist() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ironbacked Archivist",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Move {
            what: target_filtered(SelectionRequirement::Any),
            to: ZoneDest::Exile,
        })],
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
    }
}

/// Lorehold Lightspeaker — {2}{R}, 2/2 Spirit Wizard Haste.
/// Synthesised Oracle: "Haste. Whenever this creature attacks, this
/// creature deals 1 damage to any target." 3-mana hasty ping body.
pub fn lorehold_lightspeaker() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Lightspeaker",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(1),
        })],
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
    }
}

/// Lorehold Warpriest — {3}{R}{W}, 3/4 Spirit Cleric Vigilance + Lifelink.
/// Synthesised Oracle: "Vigilance, lifelink. When this creature enters,
/// this creature deals 2 damage to target creature." 5-mana defensive
/// finisher + ETB removal.
pub fn lorehold_warpriest() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Warpriest",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(2),
        })],
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
    }
}

/// Lorehold Emberscholar — {1}{R}{W}, 2/2 Spirit Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, this creature deals 1 damage to each
/// opponent." 3-mana drain-burn engine.
pub fn lorehold_emberscholar() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Emberscholar",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::magecraft_ping_each_opp(1)],
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
    }
}

/// Lorehold Relicbearer — {1}{R}{W}, 2/2 Spirit Cleric.
/// Synthesised Oracle: "Whenever one or more cards leave your
/// graveyard, put a +1/+1 counter on this creature." 3-mana gy-leave
/// growth engine.
pub fn lorehold_relicbearer() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Relicbearer",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Ember Sentinel — {2}{W}, 1/3 Spirit Cleric Vigilance.
/// Synthesised Oracle: "Vigilance. When this creature enters, you
/// gain 3 life." Defensive lifegain body.
pub fn lorehold_ember_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember Sentinel",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb_gain_life(3)],
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
    }
}

// ── Batch 47 (modern_decks) — Lorehold expansion ────────────────────────────

/// Lorehold Spiritbinder — {2}{R}{W}, 3/3 Spirit Cleric. Synthesised
/// Oracle: "When this creature enters, create a 2/2 R/W Spirit creature
/// token." 4-mana double-body wide play.
pub fn lorehold_spiritbinder() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritbinder",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Sparkflinger — {1}{R}, 2/2 Human Wizard. Synthesised Oracle:
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// this creature deals 1 damage to any target." 2-mana ping-engine
/// magecraft body. Mirror of Prismari Pyrowriter at the Lorehold slot.
pub fn lorehold_sparkflinger() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkflinger",
        cost: cost(&[generic(1), r()]),
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
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Battle Cry — {R}{W} Sorcery. Synthesised Oracle:
/// "Create a 2/2 red and white Spirit creature token with haste."
/// 2-mana Spirit-token enabler with built-in haste.
pub fn lorehold_battle_cry() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battle Cry",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: crate::effect::shortcut::create_token_with_keyword(
            PlayerRef::You,
            1,
            lorehold_spirit_token(),
            Keyword::Haste,
            Duration::EndOfTurn,
        ),
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Battle Memorial — {3}{R}{W} Sorcery. Synthesised Oracle:
/// "Lorehold Battle Memorial deals 3 damage to target creature and 3
/// damage to target player." 5-mana split-shot — slot 0 = creature
/// target, slot 1 = player target.
pub fn lorehold_battle_memorial() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battle Memorial",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(3),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Player,
                },
                amount: Value::Const(3),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Veteran — {2}{R}, 3/2 Spirit Soldier Haste.
/// Synthesised Oracle: "Haste. When this creature enters, it deals
/// 1 damage to any target." 3-mana aggressive haste body with ETB ping.
pub fn lorehold_veteran() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Veteran",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Scrollwarden — {3}{R}{W}, 3/4 Spirit Soldier Flying.
/// Synthesised Oracle: "Flying. When this creature enters, create a
/// 2/2 R/W Spirit creature token."
pub fn lorehold_scrollwarden() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Scrollwarden",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

// ── Batch 48 (modern_decks) — Lorehold expansion ────────────────────────────

/// Lorehold Flameherald II — {1}{R}, 2/1 Spirit Wizard Haste.
/// Synthesised Oracle: "Haste. When this creature enters, it deals
/// 1 damage to any target." 2-mana hasty ETB-ping body.
pub fn lorehold_flameherald_v2() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Flameherald II",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Spirit Bardguard — {2}{W}, 2/3 Spirit Soldier Vigilance.
/// Synthesised Oracle: "Vigilance." Vanilla 3-mana defensive Spirit
/// — stacks with Lorehold Anthemist / Spirit Cantor anthems and
/// Quintorius Field Historian's body of work.
pub fn spirit_bardguard() -> CardDefinition {
    CardDefinition {
        name: "Spirit Bardguard",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Sparkwarden — {1}{R}{W}, 2/2 Spirit Cleric Lifelink.
/// Synthesised Oracle: "Lifelink. Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, this creature gets +1/+0 until
/// end of turn." 3-mana lifelink scaler.
pub fn lorehold_sparkwarden() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkwarden",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Spiritscribe — {3}{R}{W} Sorcery. Synthesised Oracle:
/// "Create two 2/2 R/W Spirit creature tokens. Lorehold Spiritscribe
/// deals 1 damage to each opponent." 5-mana go-wide + drain finisher.
pub fn lorehold_spiritscribe() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritscribe",
        cost: cost(&[generic(3), r(), w()]),
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
                definition: lorehold_spirit_token(),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Phoenix-Soldier — {2}{R}{W}, 2/2 Spirit Phoenix Flying
/// + Haste. Synthesised Oracle: "Flying, haste." 4-mana double-keyword
///   evasive aggressive body — slots into Lorehold Spirit tribal shells.
pub fn lorehold_phoenix_soldier() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Phoenix-Soldier",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Phoenix],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

// ── Batch 48 follow-up (modern_decks) — Lorehold expansion 2 ────────────────

/// Spirit Spellsmith — {1}{R}{W}, 2/3 Spirit Wizard. Synthesised
/// Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, you gain 1 life." 3-mana magecraft lifegain body.
pub fn spirit_spellsmith() -> CardDefinition {
    CardDefinition {
        name: "Spirit Spellsmith",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
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
    }
}

/// Lorehold Glimmercaller — {2}{R}, 2/2 Spirit Wizard. Synthesised
/// Oracle: "When this creature enters, it deals 2 damage to target
/// creature." 3-mana ETB-burn body.
pub fn lorehold_glimmercaller() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Glimmercaller",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Refrain — {R}{W} Instant. Synthesised Oracle: "Lorehold
/// Refrain deals 2 damage to any target. You gain 2 life." 2-mana
/// flexible burn-and-lifegain.
pub fn lorehold_refrain() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Refrain",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Spirit Banner-Bearer — {2}{W}, 1/3 Spirit Soldier. Synthesised
/// Oracle: "Other Spirit creatures you control get +1/+0."
/// 3-mana Spirit-tribal anthem.
pub fn spirit_banner_bearer() -> CardDefinition {
    CardDefinition {
        name: "Spirit Banner-Bearer",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![crate::effect::StaticAbility {
            description: "Other Spirit creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Battle Drum — {2}{R}{W} Sorcery. Synthesised Oracle:
/// "Each creature you control gets +1/+0 and gains haste until end
/// of turn." 4-mana go-wide swing-turn anthem.
pub fn lorehold_battle_drum() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battle Drum",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 48 follow-up #2 (modern_decks) — more Lorehold cards ──────────────

/// Spirit Spearmaiden — {1}{W}, 2/2 Spirit Soldier. Synthesised
/// Oracle: "First strike." 2-mana aggressive first-striker.
pub fn spirit_spearmaiden() -> CardDefinition {
    CardDefinition {
        name: "Spirit Spearmaiden",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Lavabolt — {1}{R} Instant. Synthesised Oracle: "Lorehold
/// Lavabolt deals 3 damage to any target." 2-mana Lightning Bolt clone.
pub fn lorehold_lavabolt() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Lavabolt",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(3),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 49 (modern_decks) — more Lorehold cards ───────────────────────────

/// Lorehold Skyrunner — {R}{W}, 2/1 Spirit Soldier Flying + Haste.
/// Synthesised Oracle: "Flying, haste." 2-mana evasive Spirit beater —
/// the canonical Lorehold drop-and-swing turn-3 attacker.
pub fn lorehold_skyrunner() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skyrunner",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Stoneward — {2}{W}, 1/4 Spirit Cleric Vigilance.
/// Synthesised Oracle: "Vigilance. When this creature enters, target
/// creature gets +0/+2 until end of turn." Defensive Lorehold 3-drop
/// that helps a friendly creature survive combat.
pub fn lorehold_stoneward() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Stoneward",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(0),
                toughness: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyremender (v2) — {1}{R}, 2/2 Spirit Wizard.
/// Synthesised Oracle: "When this creature enters, it deals 1 damage
/// to any target." 2-mana ping-on-arrival body — pairs with the
/// Lorehold spell-cast tribe.
pub fn lorehold_pyremender_v2() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyremender Embershade",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyreward — {R}{W} Instant. Synthesised Oracle:
/// "Lorehold Pyreward deals 2 damage to any target. You gain 1 life."
/// 2-mana cheap Lightning Helix variant — Lorehold's bread-and-butter
/// burn-and-stabilize instant.
pub fn lorehold_pyreward() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyreward",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::GainLife {
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Spirit Honor Guard — {2}{R}{W}, 2/3 Spirit Soldier Vigilance + First
/// Strike. Synthesised Oracle: "Vigilance, first strike." 4-mana
/// defensive Spirit body — survives combat against most attackers and
/// keeps blocking after attacking.
pub fn spirit_honor_guard() -> CardDefinition {
    CardDefinition {
        name: "Spirit Honor Guard",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Smiteseer — {2}{R}{W}, 3/3 Spirit Cleric. Synthesised
/// Oracle: "When this creature enters, it deals 2 damage to target
/// creature. You gain 2 life." 4-mana value body — Lightning Helix
/// stapled to a 3/3.
pub fn lorehold_smiteseer() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Smiteseer",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
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
                Effect::DealDamage {
                    to: target_filtered(SelectionRequirement::Creature),
                    amount: Value::Const(2),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
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
    }
}

// ── Batch 50: Lorehold synthesised cards ───────────────────────────────────

/// Lorehold Embersmith — {R}, 1/1 Spirit Wizard Haste. Magecraft
/// deals 1 damage to any target. Cheapest hasty magecraft burner.
pub fn lorehold_embersmith() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Embersmith",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Spirit Mentor — {1}{W}, 1/3 Spirit Cleric Vigilance. Magecraft
/// gain 1 life. 2-mana scaling defensive lifegain.
pub fn spirit_mentor() -> CardDefinition {
    CardDefinition {
        name: "Spirit Mentor",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
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
    }
}

/// Lorehold Wargist — {2}{R}, 3/2 Spirit Warrior. ETB deals 1
/// damage to each opponent. 3-mana drain-equivalent ping body.
pub fn lorehold_wargist() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Wargist",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
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
    }
}

/// Lorehold Sparkstrike v2 — {R}, Instant. Deals 2 damage to target
/// creature. Cheap creature-only burn at 1 mana.
/// (Disambiguated from existing batch's Sparkstrike.)
pub fn lorehold_sparkstrike_b50() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkstrike Burst",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Spirit Battlemaster — {3}{R}{W}, 4/4 Spirit Soldier First Strike.
/// Magecraft self-pump +1/+0 EOT. 5-mana combat-ready scaling
/// magecraft body.
pub fn spirit_battlemaster() -> CardDefinition {
    CardDefinition {
        name: "Spirit Battlemaster",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Memoriam — {2}{R}{W}, Sorcery. Mints 2 Spirit tokens +
/// gain 2 life. 4-mana mint + lifegain swing.
pub fn lorehold_memoriam() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Memoriam",
        cost: cost(&[generic(2), r(), w()]),
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
                definition: lorehold_spirit_token(),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Spirit Berserker — {1}{R}, 2/1 Spirit Warrior Trample + Haste.
/// 2-mana hasty trampler — cheapest aggressive Spirit.
pub fn spirit_berserker() -> CardDefinition {
    CardDefinition {
        name: "Spirit Berserker",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste, Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Memorialist v2 — {2}{W}, 1/3 Human Cleric Vigilance. ETB
/// returns target creature card from your graveyard to your hand.
/// 3-mana defensive value body.
/// (Disambiguated from extras's Memorialist.)
pub fn lorehold_memorialist_b50() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Memorialist Adept",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Hand(PlayerRef::You),
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
    }
}

/// Lorehold Echocaller — {1}{R}{W}, 2/2 Spirit Cleric. ETB mint a
/// Spirit token + gain 1 life. 3-mana double-payoff ETB body.
pub fn lorehold_echocaller() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Echocaller",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
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
    }
}

/// Lorehold Sparkshock — {1}{R}, Instant. Seq(DealDamage 2 to any
/// target + Scry 1). 2-mana shock + smoothing.
pub fn lorehold_sparkshock() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkshock",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Skystorm — {2}{R}{W}, Sorcery. DealDamage 2 to each
/// creature opp controls + GainLife 2.
pub fn lorehold_skystorm() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skystorm",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                amount: Value::Const(2),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Reverence — {1}{R}{W}, 2/3 Spirit Cleric Vigilance. ETB
/// mints a 2/2 R/W Spirit token.
pub fn lorehold_reverence() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Reverence",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Pyromentor — {2}{R}, 2/3 Spirit Cleric. Magecraft 1
/// damage to any target. Cheaper Storm-Kiln-style ping.
pub fn lorehold_pyromentor() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyromentor",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Spirit Veteran — {3}{R}{W}, 4/4 Spirit Soldier
/// Vigilance. ETB +1/+1 counter on each other Spirit you control.
pub fn lorehold_spirit_veteran() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit Veteran",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::OtherThanSource),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Embermend — {1}{W}, Instant. Seq(GainLife 3 + Scry 1).
/// 2-mana defensive lifegain + smoothing.
pub fn lorehold_embermend() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Embermend",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spiritchron — {R}{W}, 2/2 Spirit Cleric. Magecraft puts
/// a +1/+1 counter on each Spirit you control. Disambiguated from the
/// existing `lorehold_memorialist` and `lorehold_spiritscribe`
/// factories.
pub fn lorehold_spiritchron() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritchron",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit)),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Sparklock — {2}{R}, Sorcery. DealDamage 4 to target
/// creature + Scry 1. Compact creature-removal burn at 3 mana.
pub fn lorehold_sparklock() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparklock",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(4),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── batch 53: more Lorehold cards ───────────────────────────────────────────

/// Lorehold Emberscribe II — {1}{R}, 2/2 Spirit Wizard Haste. Magecraft
/// pings any target for 1.
pub fn lorehold_emberscribe_v2() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Emberscribe II",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Spirit Redeemer — {2}{W}, 2/3 Spirit Cleric Vigilance + Lifelink.
/// Defensive sticky lifelink anchor.
pub fn lorehold_spirit_redeemer() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit Redeemer",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Emberlock — {R}{W}, Instant. Seq(DealDamage 2 any + GainLife 2).
/// 2-mana Lightning Helix template.
pub fn lorehold_emberlock() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Emberlock",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Skyblaze — {2}{R}{W}, Sorcery. Seq(CreateToken 1 Spirit +
/// DealDamage 2 to each opponent creature). 4-mana wide anti-creature
/// burn + Spirit body.
pub fn lorehold_skyblaze() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skyblaze",
        cost: cost(&[generic(2), r(), w()]),
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
                definition: lorehold_spirit_token(),
            },
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Spirit Pyromage — {1}{R}, 2/2 Spirit Wizard Haste. Aggressive vanilla
/// Spirit. Disambiguated from existing `spirit_pyremage`.
pub fn spirit_blazekin() -> CardDefinition {
    CardDefinition {
        name: "Spirit Blazekin",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

// ── batch 54: more Lorehold cards ───────────────────────────────────────────

/// Lorehold Invoker — {2}{R}, 3/2 Spirit Cleric Haste. Magecraft ping each
/// opp for 1.
pub fn lorehold_invoker() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_each_opp;
    CardDefinition {
        name: "Lorehold Invoker",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_each_opp(1)],
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
    }
}

/// Spirit Sparkmage — {R}{W}, 2/2 Spirit Cleric. ETB Lightning Helix
/// template (deal 2 to any target + gain 2 life).
pub fn spirit_sparkmage() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Spirit Sparkmage",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]))],
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
    }
}

/// Lorehold Chronicler v2 — {1}{R}{W}, 2/2 Spirit Wizard Flying. Magecraft
/// self-pump +1/+1 EOT. (Disambiguated from the existing
/// `extras::lorehold_chronicler` which is a different shell.)
pub fn lorehold_chronicler_v2() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Chronicler Aerist",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Relicwarden — {3}{R}{W}, 3/4 Spirit Soldier Vigilance. ETB
/// puts a +1/+1 counter on each other Spirit you control (Spirit tribal).
pub fn lorehold_relicwarden() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Relicwarden",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 55): 5 more Lorehold cards ───────────────────

/// Lorehold Pyrescribe Elder — {1}{R}{W}, 2/2 Spirit Wizard. Magecraft
/// Seq(deal 1 to any target + GainLife 1). Lorehold Apprentice-style
/// scaling burn+lifegain in a slightly larger frame.
pub fn lorehold_pyrescribe_elder() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrescribe Elder",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
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
    }
}

/// Lorehold Skirmish II — {2}{R}, Sorcery. Creates a 2/2 R/W Spirit
/// token with Haste EOT (Lorehold Skirmish-template). Pairs with
/// attack-trigger payoffs.
pub fn lorehold_skirmish_v2() -> CardDefinition {
    use crate::effect::shortcut::create_token_with_keyword;
    CardDefinition {
        name: "Lorehold Skirmish II",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: create_token_with_keyword(
            PlayerRef::You,
            1,
            lorehold_spirit_token(),
            Keyword::Haste,
            Duration::EndOfTurn,
        ),
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Sparkflame — {R}, Instant. Deal 2 damage to any target.
/// Compact Shock-template at the {R} slot — slots into Lorehold burn-lean
/// shells.
pub fn lorehold_sparkflame() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkflame",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spiritcaller II — {2}{R}{W}, 3/3 Spirit Cleric. ETB mints
/// 2 R/W Spirit tokens. 4-mana go-wide Lorehold body.
pub fn lorehold_spiritcaller_b55() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritcaller II",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 2)],
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
    }
}

/// Spirit Banneret — {R}{W}, 2/2 Spirit Soldier. Static "Other Spirit
/// creatures you control get +1/+0." Spirit-tribal anthem.
pub fn spirit_banneret() -> CardDefinition {
    use crate::effect::StaticAbility;
    CardDefinition {
        name: "Spirit Banneret",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Spirit creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 56) — new Lorehold STX cards ─────────────────

/// Lorehold Forge-Cleric — {1}{R}{W}, 2/3 Spirit Cleric Vigilance.
/// Magecraft → put a +1/+1 counter on a target friendly Spirit. A
/// Spirit-tribal magecraft scaler that pairs with Spirit Banneret /
/// Quintorius for chained +1/+1 growth on the team.
pub fn lorehold_forge_cleric() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Forge-Cleric",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyrescholar II — {2}{R}, 2/2 Spirit Wizard Haste.
/// Magecraft: deals 2 damage to target opponent. A direct burn
/// magecraft body — strict upgrade to a 2-mana Pyrescribe at the
/// {2}{R} slot.
pub fn lorehold_pyrescholar_b56() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrescholar II",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(2),
        })],
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
    }
}

/// Lorehold Summit — {3}{R}{W}, Sorcery. Mint 2 Spirit tokens and
/// give each creature you control Haste until end of turn. Lorehold
/// alpha-strike top-end — mints two bodies + turns the team into a
/// surprise attack.
pub fn lorehold_summit() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Summit",
        cost: cost(&[generic(3), r(), w()]),
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
                definition: lorehold_spirit_token(),
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Spirit Scribe — {1}{W}, 1/3 Spirit Cleric. ETB Scry 2. Defensive
/// smoothing body + Spirit-tribal type — fuels Lorehold Phantasmist,
/// Spirit Banneret, Quintorius.
pub fn spirit_scribe() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Spirit Scribe",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
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
    }
}

/// Lorehold Ember-Strike — {R}, Instant. Deal 1 damage to any target
/// + Surveil 1. 1-mana ping + selection — feeds magecraft and Lorehold
///   graveyard payoffs.
pub fn lorehold_ember_strike() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Strike",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            },
            Effect::Surveil {
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 57): 4 more Lorehold cards ───────────────────

/// Lorehold Battlepriest — {2}{W}, 2/3 Spirit Cleric with Lifelink.
/// Magecraft gain 1 life. 3-mana lifelink body with on-cast lifegain
/// scaling — pairs nicely with Inkrise Lifedrainer / Light of Promise.
pub fn lorehold_battlepriest() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battlepriest",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
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
    }
}

/// Lorehold Bonereader — {1}{R}{W}, 2/2 Spirit Cleric with Vigilance.
/// Magecraft exile target card from a graveyard. 3-mana defensive
/// magecraft engine + recurring graveyard hate.
pub fn lorehold_bonereader_b57() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Bonereader II",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Move {
            what: target_filtered(SelectionRequirement::Any),
            to: ZoneDest::Exile,
        })],
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
    }
}

/// Lorehold Sparkscholar — {1}{R}, 2/2 Spirit Wizard with Haste.
/// Magecraft 1 damage to target creature. 2-mana hasty magecraft body
/// — pure creature-removal engine in spell-heavy shells.
pub fn lorehold_sparkscholar_b57() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkscholar II",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(1),
        })],
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
    }
}

/// Lorehold Reverence II — {2}{R}{W}, 3/3 Spirit Cleric. ETB Seq(mint
/// 1 R/W Spirit + GainLife 2). 4-mana double-payoff body — wide-and-
/// defensive Spirit anchor that scales with Quintorius / Tenured anthems.
pub fn lorehold_reverence_v2() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Reverence II",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]))],
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
    }
}

// ── Push (modern_decks, batch 58): 4 more Lorehold cards ───────────────────

/// Lorehold Skybattler — {R}{W}, 2/2 Spirit Soldier with Flying.
/// 2-mana evasive body — clean aggressive flier in the Lorehold
/// Spirits package.
pub fn lorehold_skybattler() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skybattler",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Bonechanter — {1}{R}, 2/1 Spirit Wizard with Haste.
/// Magecraft: target creature gains Menace EOT. Combat trickster
/// that turns the Lorehold IS chain into unblockable attackers.
pub fn lorehold_bonechanter() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Lorehold Bonechanter",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::GrantKeyword {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::Menace,
            duration: Duration::EndOfTurn,
        })],
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
    }
}

/// Lorehold Sparkdancer — {2}{R}{W}, 2/3 Spirit Wizard. ETB Seq(2 dmg
/// any target + GainLife 2). 4-mana double-payoff value engine — Bolt
/// + tonic for the Lorehold midrange shell.
pub fn lorehold_sparkdancer() -> CardDefinition {
    use crate::effect::shortcut::{etb, target_filtered};
    CardDefinition {
        name: "Lorehold Sparkdancer",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]))],
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
    }
}

/// Lorehold Reliquarian — {3}{R}{W}, 3/4 Spirit Cleric with Vigilance.
/// ETB mint 1 R/W Spirit token. Magecraft: gain 1 life. 5-mana wide
/// anchor that combines body + token + recurring incidental life.
pub fn lorehold_reliquarian() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Reliquarian",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
            }),
            magecraft_gain_life(1),
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
    }
}

// ── Push (modern_decks, batch 59): 5 more Lorehold cards ───────────────────

/// Lorehold Skyignite — {R}{W}, 2/1 Spirit Soldier with Flying + First
/// Strike. Magecraft: 1 damage to any target. 2-mana double-keyword
/// evasive pinger.
pub fn lorehold_skyignite() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skyignite",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Pyrelearner — {1}{R}, 2/1 Spirit Wizard with Haste.
/// Magecraft self-pump +1/+0 EOT. Aggressive hasty body that grows on
/// each IS cast.
pub fn lorehold_pyrelearner() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrelearner",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Spiritbinder II — {2}{R}{W}, 2/3 Spirit Cleric with Vigilance.
/// ETB Seq(mint a 2/2 Spirit token + gain 1 life). 4-mana wide
/// double-payoff body.
pub fn lorehold_spiritbinder_b59() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Spiritbinder II",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
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
    }
}

/// Lorehold Emberscribe II — {1}{R}{W}, 2/2 Spirit Wizard. Magecraft: deal
/// 1 damage to any target. 3-mana Lorehold ping engine.
pub fn lorehold_emberscribe_b59() -> CardDefinition {
    // Renamed from "Lorehold Emberscribe II" to "(b59)" to disambiguate
    // from the b30 variant `lorehold_emberscribe_v2`.
    CardDefinition {
        name: "Lorehold Emberscribe (b59)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

// ── Push (modern_decks, batch 60): 3 more Lorehold cards ───────────────────

/// Lorehold Chronicler III — {2}{W}, 2/3 Spirit Cleric with Vigilance. ETB
/// mint a Spirit token. 3-mana wide body + flier-friendly defender.
pub fn lorehold_chronicler_b60() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Chronicler III",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Sparkmage II — {1}{R}, 2/2 Spirit Wizard with Haste.
/// Magecraft: 1 damage to any target. 2-mana hasty pinger body —
/// canonical aggressive Lorehold ping shape.
pub fn lorehold_sparkmage_b60() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkmage II",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Battle-Sage — {2}{R}{W}, 3/3 Spirit Soldier with First
/// Strike. Magecraft: target friendly creature gets +1/+1 EOT. 4-mana
/// combat-anchor + per-cast pump.
pub fn lorehold_battle_sage() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Lorehold Battle-Sage",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            1, 1,
        )],
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
    }
}

/// Lorehold Relicseer — {3}{R}{W}, 3/3 Spirit Wizard with Flying. ETB
/// exile target card from a graveyard. 5-mana evasive body + graveyard
/// hate.
pub fn lorehold_relicseer() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Relicseer",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::take(
                Selector::CardsInZone {
                    who: PlayerRef::EachOpponent,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Any,
                },
                Value::Const(1),
            ),
            to: ZoneDest::Exile,
        })],
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
    }
}

// ── Push (modern_decks, batch 61): 5 more Lorehold cards ────────────────────

/// Lorehold Emberspeaker — {2}{R}, 2/2 Spirit Wizard Haste. ETB deal 1
/// damage to any target. 3-mana ping-on-entry haste body. Uses the
/// `etb_ping_any(1)` shortcut helper.
pub fn lorehold_emberspeaker() -> CardDefinition {
    use crate::effect::shortcut::etb_ping_any;
    CardDefinition {
        name: "Lorehold Emberspeaker",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_ping_any(1)],
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
    }
}

/// Lorehold Battle-Keeper — {2}{R}{W}, 3/3 Spirit Cleric Vigilance. ETB
/// Seq(mint a 2/2 R/W Spirit token + deal 1 damage to any target). 4-mana
/// defensive evasive token-mint engine + ping rider.
pub fn lorehold_battle_keeper() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Battle-Keeper",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
            },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            },
        ]))],
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
    }
}

/// Spirit Bannerer — {1}{W}, 1/2 Spirit Cleric. Magecraft: each Spirit
/// you control gets +1/+0 EOT (`ForEach Spirit/ControlledByYou →
/// PumpPT(+1/+0, EOT)`). 2-mana Spirit-tribal magecraft engine.
pub fn spirit_bannerer() -> CardDefinition {
    use crate::effect::shortcut::magecraft_pump_each_creature_type;
    CardDefinition {
        name: "Spirit Bannerer",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_pump_each_creature_type(
            CreatureType::Spirit,
            1,
            0,
        )],
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
    }
}

/// Lorehold Scholar II — {1}{R}{W}, 2/2 Spirit Cleric. Magecraft
/// GainLife 1. 3-mana lifegain-on-cast body.
pub fn lorehold_scholar_b61() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Scholar II",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
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
    }
}

/// Lorehold Warpoet — {3}{R}{W}, 3/3 Spirit Soldier First Strike +
/// Lifelink. ETB mints a 2/2 R/W Spirit token. 5-mana evasive combat-
/// keyword + token-mint finisher.
pub fn lorehold_warpoet() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Warpoet",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

// ── Push (modern_decks, batch 62): 2 more Lorehold cards ────────────────────

/// Lorehold Brimstoner — {3}{R}, 3/2 Spirit Wizard Haste. ETB 2 damage
/// any target via the new `etb_ping_any(2)` shortcut. 4-mana hasty
/// burn-on-entry body.
pub fn lorehold_brimstoner() -> CardDefinition {
    use crate::effect::shortcut::etb_ping_any;
    CardDefinition {
        name: "Lorehold Brimstoner",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_ping_any(2)],
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
    }
}

/// Spirit Reliquarian — {1}{R}{W}, 2/3 Spirit Cleric Vigilance. Static
/// "Other Spirit creatures you control get +1/+0" — Spirit-tribal
/// anthem at the 3-mana slot. Mirrors Spirit Banneret on a bigger frame
/// with a vigilance keyword for the alpha-strike-into-defense turn.
pub fn spirit_reliquarian() -> CardDefinition {
    use crate::effect::StaticAbility;
    CardDefinition {
        name: "Spirit Reliquarian",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Spirit creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 63): 5 more Lorehold cards ────────────────────

/// Spirit Sparkblade — {1}{R}, 2/2 Spirit Warrior Haste. Magecraft +1/+0
/// EOT self-pump. 2-mana hasty aggressive Spirit. Stacks with Spirit
/// anthems for early pressure.
pub fn spirit_sparkblade() -> CardDefinition {
    CardDefinition {
        name: "Spirit Sparkblade",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Spiritchron II — {2}{R}{W}, 3/3 Spirit Cleric Vigilance.
/// ETB Seq(mint 2 Spirit tokens). 4-mana go-wide Spirit anchor.
pub fn lorehold_spiritchron_b63() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritchron II",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 2)],
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
    }
}

/// Lorehold Embertongue — {R}{W}, Instant. Seq(DealDamage 2 to any target +
/// GainLife 1). 2-mana Lightning-Helix-template at half power. Affordable
/// early burn + life-swing.
pub fn lorehold_embertongue() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Embertongue",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::GainLife {
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Sparkstoneflinger — {2}{R}, 2/3 Spirit Wizard. Magecraft
/// 1 damage to any target. 3-mana sturdier magecraft burn engine.
pub fn lorehold_sparkstoneflinger() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkstoneflinger",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Memorialcaller — {3}{R}{W}, 3/4 Spirit Cleric Lifelink. ETB
/// mints 2 Spirit tokens + magecraft gain 1 life. 5-mana sticky lifelink
/// + token-mint payoff.
pub fn lorehold_memorialcaller() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Memorialcaller",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                definition: lorehold_spirit_token(),
                count: Value::Const(2),
            }),
            magecraft_gain_life(1),
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
    }
}

/// Lorehold Sparkscholar III — {1}{R}, 2/1 Spirit Wizard Haste. ETB
/// deals 1 damage to target creature. Uses the new `etb_ping_creature`
/// shortcut. 2-mana hasty creature-removal ETB body.
pub fn lorehold_sparkscholar_b63() -> CardDefinition {
    use crate::effect::shortcut::etb_ping_creature;
    CardDefinition {
        name: "Lorehold Sparkscholar III",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_ping_creature(1)],
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
    }
}

/// Lorehold Sparkscholar IV — {2}{R}, 2/2 Spirit Wizard. Magecraft 1
/// damage to target creature via the new `magecraft_ping_creature`
/// shortcut. Creature-removal-only magecraft body at the 3-mana slot.
pub fn lorehold_sparkscholar_b63_v2() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_creature;
    CardDefinition {
        name: "Lorehold Sparkscholar IV",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_creature(1)],
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
    }
}

/// Lorehold Coinflinger — {2}{R}, 2/2 Spirit Wizard. "When this creature
/// enters, flip a coin. If you win the flip, this creature deals 3
/// damage to any target. If you lose, you discard a card." CR 705 +
/// CR 122 — exercises the new `Effect::FlipCoin` primitive on a
/// representative red gamble body.
pub fn lorehold_coinflinger() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Coinflinger",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::FlipCoin {
            count: Value::Const(1),
            on_heads: Box::new(Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            }),
            on_tails: Box::new(Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
            }),
        })],
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
    }
}

// ── Push (modern_decks, batch 64): 4 more Lorehold cards ───────────────────

/// Lorehold Ember-Speaker (batch 64) — {1}{R}, 2/2 Spirit Wizard. ETB
/// ping 2 to any target. 2-mana burst body via the `etb_ping_any(2)`
/// shortcut.
pub fn lorehold_ember_speaker_b64() -> CardDefinition {
    use crate::effect::shortcut::etb_ping_any;
    CardDefinition {
        name: "Lorehold Ember-Speaker (b64)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_ping_any(2)],
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
    }
}

/// Spirit Spellblade — {2}{R}{W}, 3/3 Spirit Soldier First Strike +
/// Vigilance. 4-mana aggressive Spirit body for Lorehold tribal shells.
pub fn spirit_spellblade() -> CardDefinition {
    CardDefinition {
        name: "Spirit Spellblade",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Sparkchorus — {3}{R}{W}, Sorcery. Seq(mint 2 Spirit tokens +
/// ping 2 to any target). 5-mana go-wide + burn finisher.
pub fn lorehold_sparkchorus() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkchorus",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: lorehold_spirit_token(),
                count: Value::Const(2),
            },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Sigilbearer — {2}{W}, 2/3 Spirit Cleric Vigilance. Magecraft
/// gain 1 life. 3-mana defensive body with on-cast lifegain.
pub fn lorehold_sigilbearer() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sigilbearer",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
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
    }
}

// ── Push (modern_decks, batch 66): 6 more Lorehold cards ───────────────────

/// Spirit Wardancer — {2}{W}, 2/2 Spirit Soldier Vigilance. Magecraft
/// +1/+1 EOT self-pump via the `magecraft_self_pump(1, 1)` shortcut.
pub fn spirit_wardancer() -> CardDefinition {
    CardDefinition {
        name: "Spirit Wardancer",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyromancer — {1}{R}{W}, 2/2 Spirit Wizard Haste. ETB ping 2
/// to any target via the `etb_ping_any(2)` shortcut.
pub fn lorehold_pyromancer_b66() -> CardDefinition {
    use crate::effect::shortcut::etb_ping_any;
    CardDefinition {
        name: "Lorehold Pyromancer (b66)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_ping_any(2)],
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
    }
}

/// Lorehold Spiritmint (batch 66) — {2}{R}, 2/2 Spirit Wizard. ETB mint
/// 1 Spirit token. 3-mana double-body for Lorehold tribal shells.
pub fn lorehold_spiritmint_b66() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Spiritmint (b66)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: lorehold_spirit_token(),
            count: Value::Const(1),
        })],
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
    }
}

/// Lorehold Battlegrave — {3}{R}{W}, 3/4 Spirit Soldier First Strike +
/// Vigilance. ETB return target Creature card from your gy → bf
/// untapped. 5-mana reanimator finisher.
pub fn lorehold_battlegrave() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Battlegrave",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature,
            }),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        })],
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
    }
}

/// Lorehold Skybearer — {2}{W}, 2/3 Spirit Cleric Flying + Vigilance.
/// 3-mana evasive defensive body for Lorehold/Silverquill shells.
pub fn lorehold_skybearer() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skybearer",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Spellbreaker — {1}{R}, 2/1 Spirit Wizard. Magecraft ping 1
/// to any target via `magecraft_ping_any(1)`. 2-mana magecraft burn.
pub fn lorehold_spellbreaker() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spellbreaker",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

// ── Push (modern_decks, batch 67): 6 more Lorehold cards ───────────────────

/// Lorehold Sparkscholar (b67) — {1}{R}{W}, 2/2 Spirit Wizard First
/// Strike. Magecraft ping 1 to any target. 3-mana first-strike
/// magecraft ping.
pub fn lorehold_sparkscholar_b67() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkscholar (b67)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Cinderpriest (b67) — {2}{R}{W}, 3/3 Spirit Cleric. ETB
/// drain 1 + magecraft +1/+0 self-pump. 4-mana lifegain + scaling body.
pub fn lorehold_cinderpriest_b67() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Cinderpriest (b67)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(1), magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Memorialer — {2}{W}, 2/3 Spirit Cleric Vigilance. ETB
/// returns target IS card from your gy → hand. 3-mana value reanimator.
pub fn lorehold_memorialer() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Memorialer",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            }),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
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
    }
}

/// Lorehold Spiritflare — {R}{W}, Instant. Deals 2 damage to any
/// target and you gain 2 life. 2-mana drain-burn template.
pub fn lorehold_spiritflare() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritflare",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spirit-Crier — {2}{R}, 3/2 Spirit Warrior Haste. Dies
/// trigger mints a 2/2 R/W Spirit. 3-mana hasty trade-up body.
pub fn lorehold_spirit_crier() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Crier",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                definition: lorehold_spirit_token(),
                count: Value::Const(1),
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
    }
}

/// Lorehold Bellringer — {3}{R}{W}, 4/3 Spirit Cleric Haste. ETB mints
/// 1 Spirit token. 5-mana fast double-body finisher.
pub fn lorehold_bellringer() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Bellringer",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: lorehold_spirit_token(),
            count: Value::Const(1),
        })],
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
    }
}

// ── Push (modern_decks, batch 68): more Lorehold R/W cards ──────────────

/// Lorehold Sparkshrine — {2}{R}{W}, Sorcery. Seq(DealDamage 2 to any
/// target + CreateToken Spirit). 4-mana burn + body.
pub fn lorehold_sparkshrine() -> CardDefinition {
    use crate::card::SelectionRequirement;
    CardDefinition {
        name: "Lorehold Sparkshrine",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: lorehold_spirit_token(),
                count: Value::Const(1),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Embertenured — {1}{R}{W}, 2/3 Spirit Cleric Vigilance.
/// Magecraft +1/+0 EOT self-pump. 3-mana vigilance magecraft body.
pub fn lorehold_embertenured() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Embertenured",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Spirit Glyphbinder — {2}{W}, 2/3 Spirit Cleric. ETB +1/+1 counter
/// on another target creature you control. 3-mana sticky pumper.
pub fn spirit_glyphbinder() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Spirit Glyphbinder",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou)),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyrebinder — {1}{R}, 2/2 Spirit Wizard. ETB DealDamage 2
/// to target creature. 2-mana ETB ping body.
pub fn lorehold_pyrebinder() -> CardDefinition {
    use crate::effect::shortcut::etb_ping_creature;
    CardDefinition {
        name: "Lorehold Pyrebinder",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_ping_creature(2)],
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
    }
}

/// Lorehold Heroic Sage — {R}{W}, 2/2 Spirit Warrior First Strike +
/// Lifelink. 2-mana double-keyword Spirit attacker — Lorehold race
/// breaker on a 2-drop frame.
pub fn lorehold_heroic_sage() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Heroic Sage",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

// ── Batch 125 (push claude/modern_decks): four new Lorehold cards ──────────

/// Lorehold Bloodrazer — {2}{R}, 3/2 Spirit Warrior. "Whenever this
/// creature attacks, it deals 1 damage to any target." Attack-trigger
/// ping engine, uses the new `on_attack_ping_any` shortcut.
pub fn lorehold_bloodrazer_b125() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Bloodrazer (b125)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_ping_any(1)],
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
    }
}

/// Lorehold Saintkeeper — {2}{W}, 2/3 Spirit Cleric Vigilance.
/// "Whenever this creature attacks, you gain 1 life." Attack-trigger
/// lifegain via the new `on_attack_gain_life` shortcut.
pub fn lorehold_saintkeeper_b125() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Saintkeeper (b125)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_gain_life(1)],
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
    }
}

/// Lorehold Vanguardian — {2}{R}{W}, 3/3 Spirit Soldier. "Whenever
/// this creature attacks, each opponent loses 1 life and you gain 1
/// life." Attack-trigger drain via the new `on_attack_drain` shortcut.
pub fn lorehold_vanguardian_b125() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Vanguardian (b125)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_drain(1)],
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
    }
}

/// Lorehold Heraldcaller — {3}{R}{W}, 3/4 Spirit Cleric Flying.
/// ETB Seq(mint 2 lorehold Spirit tokens + GainLife 2). 5-mana
/// go-wide finisher with lifegain rider.
pub fn lorehold_heraldcaller_b125() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Heraldcaller (b125)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: lorehold_spirit_token(),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]))],
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
    }
}

// ── Batch 126 (push claude/modern_decks): six new Lorehold cards ──────────

/// Lorehold Spiritbinder (b126) — {2}{W}, 2/3 Spirit Cleric. "When this
/// creature dies, create a 2/2 R/W Spirit token." Self-replacing body via
/// the new `dies_mint_token` shortcut.
pub fn lorehold_spiritbinder_b126() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritbinder (b126)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Cinderscholar (b126) — {1}{R}, 2/1 Human Wizard. Magecraft
/// self-pump +1/+0 EOT. Aggressive 2-mana magecraft body.
pub fn lorehold_cinderscholar_b126() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Cinderscholar (b126)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Halfblood (b126) — {3}{R}{W}, 4/4 Spirit Soldier Trample.
/// 5-mana go-large finisher — Trample with double tribal subtypes
/// (Spirit + Soldier) for Spirit-lord shells.
pub fn lorehold_halfblood_b126() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Halfblood (b126)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Skywatcher (b126) — {2}{W}, 1/4 Spirit Cleric Flying +
/// Vigilance. 3-mana defensive double-keyword evasive blocker —
/// Tenured Inkcaster style anthem fodder for Spirit-tribal shells.
pub fn lorehold_skywatcher_b126() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skywatcher (b126)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Ember-Mage (b126) — {1}{R}, 1/2 Human Wizard. Magecraft
/// ping any 1. 2-mana magecraft burn body — Prodigal Sorcerer template
/// at instant-cast speed.
pub fn lorehold_ember_mage_b126() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Mage (b126)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Sparkscholar (b126) — {2}{R}, 2/2 Human Wizard. On_dies
/// ping 2 to any target. 3-mana parting-shot body — Mogg Fanatic at a
/// higher P/T frame using the new `dies_ping_any` shortcut.
pub fn lorehold_sparkscholar_b126() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkscholar (b126)",
        cost: cost(&[generic(2), r()]),
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
        triggered_abilities: vec![dies_ping_any(2)],
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
    }
}

// ── Batch 127 (push claude/modern_decks): six new Lorehold cards ──────────

/// Lorehold Aerialist (b127) — {1}{W}, 2/2 Spirit Cleric Flying. A
/// vanilla 2-mana evasive Spirit — Lorehold tribal fodder for the
/// Lorehold spirit anthems (Quintorius, Hofri).
pub fn lorehold_aerialist_b127() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Aerialist (b127)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Ironbound (b127) — {2}{W}, 2/4 Spirit Soldier. A defensive
/// 3-mana spirit-tribal body — pairs with the spirit anthems for stable
/// ground defense.
pub fn lorehold_ironbound_b127() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ironbound (b127)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
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
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyrebrand (b127) — {1}{R}, 1/2 Human Wizard. Magecraft
/// ping each opp 1. Lorehold variant of Pestmancer — every IS spell
/// pings each opponent for 1 free damage.
pub fn lorehold_pyrebrand_b127() -> CardDefinition {
    use crate::effect::shortcut::cast_is_instant_or_sorcery;
    CardDefinition {
        name: "Lorehold Pyrebrand (b127)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::DealDamage {
                amount: Value::Const(1),
                to: Selector::Player(PlayerRef::EachOpponent),
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
    }
}

/// Lorehold Vetenarian (b127) — {3}{R}{W}, 3/3 Spirit Cleric.
/// ETB GainLife 3 + on_attack DealDamage 2 to target opp creature via
/// `target_filtered`. A 5-mana attack-focused mid-curve threat.
pub fn lorehold_veteran_b127() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Veteran (b127)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_gain_life(3),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::DealDamage {
                    amount: Value::Const(2),
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
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
    }
}

/// Lorehold Honorbound (b127) — {1}{R}{W}, 2/3 Spirit Knight First
/// Strike. 3-mana evasive Lorehold body — early curve aggressor in
/// Spirit/Knight shells.
pub fn lorehold_honorbound_b127() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Honorbound (b127)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Embercurse (b127) — {R}{W} Sorcery. DealDamage 3 to a
/// target creature + GainLife 2. Lorehold's mini-Sacred-Fire at a
/// tighter curve (2 mana vs Sacred Fire's no-cost-burn-pump).
pub fn lorehold_embercurse_b127() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Embercurse (b127)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                amount: Value::Const(3),
                to: target_filtered(SelectionRequirement::Creature),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 128 (push claude/modern_decks): new Lorehold cards ──────────────

/// Lorehold Skybinder (b128) — {2}{W}, 2/2 Spirit Cleric Flying.
/// Magecraft +1/+1 EOT self-pump — small flying body that scales in
/// spell-heavy shells.
pub fn lorehold_skybinder_b128() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skybinder (b128)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Bookforger (b128) — {3}{R}, 3/3 Human Wizard. Magecraft
/// "create a Treasure token" — Lorehold spin on Prismari's treasure
/// generators, fueling expensive R/W finishers.
pub fn lorehold_bookforger_b128() -> CardDefinition {
    use crate::effect::shortcut::magecraft_treasure;
    CardDefinition {
        name: "Lorehold Bookforger (b128)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_treasure()],
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
    }
}

/// Lorehold Bell-Ringer (b128) — {1}{R}{W}, 2/3 Spirit Cleric. ETB
/// gain 2 life + Spirit token mints — Lorehold double-payoff 3-drop
/// (durable body + immediate Spirit anthem fuel + lifegain).
pub fn lorehold_bell_ringer_b128() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Bell-Ringer (b128)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(2), etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Cliffstrike (b128) — {2}{R}{W} Sorcery. Seq(DealDamage 4
/// target creature + GainLife 3). Larger Embercurse — 4-mana removal-
/// and-lifegain finisher (kills 4-toughness ground bodies).
pub fn lorehold_cliffstrike_b128() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Cliffstrike (b128)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                amount: Value::Const(4),
                to: target_filtered(SelectionRequirement::Creature),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Sparkmender (b128) — {2}{W}, 2/3 Spirit Cleric Lifelink.
/// 3-mana defensive lifelinker that scales with the Lorehold lifegain
/// payoffs (Light of Promise template).
pub fn lorehold_sparkmender_b128() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkmender (b128)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Battlespirit (b128) — {3}{R}{W}, 4/4 Spirit Warrior with
/// Haste. ETB mints a 2/2 R/W Spirit token via `etb_mint_token`. 5-mana
/// haste finisher with go-wide rider.
pub fn lorehold_battlespirit_b128() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battlespirit (b128)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Soulreaver (b128) — {2}{R}{W}, 3/3 Spirit Knight First
/// Strike. Magecraft ping each opp 1 — Lorehold Pyrebrand on a bigger
/// body with first strike for combat survival.
pub fn lorehold_soulreaver_b128() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Soulreaver (b128)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
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
    }
}

/// Lorehold Pyrestone (b128) — {R}{W} Instant. Target creature gets
/// +2/+0 and gains first strike EOT — Lorehold combat trick at a
/// cheap rate. Same shape as Silverquill Discipline but trades
/// lifelink for first strike for a more aggressive R/W tempo play.
pub fn lorehold_pyrestone_b128() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrestone (b128)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 129 (push claude/modern_decks): new Lorehold cards ──────────────

/// Lorehold Spirit Banner (b129) — {2}{R}{W}, 2/3 Spirit. Static
/// "Other Spirit creatures you control get +1/+1." Spirit-tribal
/// anthem mirroring Tenured Inkcaster for the Lorehold/Spirit pool
/// (Aerialist, Ironbound, Bell-Ringer, Battlespirit, Skybinder,
/// Honorbound, Veteran, Soulreaver, Sparkmender all benefit).
pub fn lorehold_spirit_banner_b129() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Lorehold Spirit Banner (b129)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Spirit creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Stoneglyph (b129) — {1}{R}{W}, 1/4 Spirit Cleric Defender.
/// Activated `{R}{W}, {T}: Lorehold Stoneglyph deals 2 damage to any
/// target.` — a 3-mana defender mana sink that pings each turn.
pub fn lorehold_stoneglyph_b129() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Stoneglyph (b129)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[r(), w()]),
            effect: Effect::DealDamage {
                amount: Value::Const(2),
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
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
    }
}

/// Lorehold Pyrespirit (b129) — {1}{R} 2/1 Spirit, Haste. Vanilla
/// red-side Spirit aggressor that benefits from the Lorehold Spirit
/// Banner anthem.
pub fn lorehold_pyrespirit_b129() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrespirit (b129)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Lectern (b129) — {3} Artifact. Static "Other Spirit
/// creatures you control have lifelink." A noncreature Spirit-tribal
/// payoff that costs no color and pairs with the Battlespirit haste
/// finisher and the Bell-Ringer ETB.
pub fn lorehold_lectern_b129() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Lorehold Lectern (b129)",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Spirit creatures you control have lifelink.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Lifelink,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Memorist (b129) — {2}{W}, 2/2 Spirit Cleric. ETB returns
/// target Spirit card with mana value ≤ 2 from your graveyard to your
/// hand. Lorehold spirit-recursion engine in a 3-mana body.
pub fn lorehold_memorist_b129() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Memorist (b129)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                    .and(SelectionRequirement::ManaValueAtMost(2)),
            }),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
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
    }
}

/// Lorehold Sparkscholar Pt II (b129) — {3}{R}{W}, 3/3 Spirit Wizard.
/// Magecraft mints a Lorehold Spirit token. Each instant or sorcery
/// you cast adds a 2/2 R/W Spirit body to the board — Lorehold's
/// answer to Inkling Penmaster but with a bigger body.
pub fn lorehold_sparkscholar_ii_b129() -> CardDefinition {
    use crate::effect::shortcut::magecraft_mint_token;
    CardDefinition {
        name: "Lorehold Sparkscholar II (b129)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Excavation (b129) — {2}{R}{W} Sorcery. Create two 2/2 R/W
/// Spirit creature tokens. Lorehold's Defend-the-Campus equivalent at
/// a cheaper rate (4 mana, 2 tokens vs Silverquill's 5 mana, 3 tokens).
pub fn lorehold_excavation_b129() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Excavation (b129)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyreverse (b129) — {1}{R} Instant. Deal 2 damage to any
/// target, gain 1 life. Lorehold's mini-Lightning Helix at a cheaper
/// rate (2 mana, 2 damage + 1 life vs Helix's 2 mana, 3 + 3).
pub fn lorehold_pyreverse_b129() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyreverse (b129)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                amount: Value::Const(2),
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
            },
            Effect::GainLife {
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Sparkmender Pt II (b129) — {3}{W}, 3/4 Spirit Cleric
/// Vigilance Lifelink. A defensive top-end body — premier Lorehold
/// race breaker, scales powerfully with the Spirit Banner anthem.
pub fn lorehold_sparkmender_ii_b129() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkmender II (b129)",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Embertongue (b129) — {2}{R}, 3/2 Human Wizard. Magecraft
/// deals 1 damage to target opp creature — narrower than `ping_any`
/// to keep the trigger creature-focused (combat support for the team).
pub fn lorehold_embertongue_b129() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Lorehold Embertongue (b129)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            amount: Value::Const(1),
            to: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
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
    }
}

// ── Batch 130 (push claude/modern_decks): more Lorehold cards ───────────────

/// Lorehold Spiritcaller (b130) — {R}{W}, 1/1 Spirit Cleric. ETB mints
/// a 2/2 R/W Spirit token via the shared `lorehold_spirit_token`. A
/// 2-mana mint that snowballs Spirit tribal pools (Banner, Lectern,
/// Sparkscholar II) and lifts the Lorehold curve at the 2-drop slot.
pub fn lorehold_spiritcaller_b130() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritcaller (b130)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Skyguard Banner (b130) — {3}{W}, 2/3 Spirit Soldier. Static
/// "Other Spirit creatures you control have flying." A second Lorehold
/// Spirit-tribal anthem that swaps the Banner's +1/+1 for evasion —
/// pairs with the Banner to swing in over Reach defenders.
pub fn lorehold_skyguard_banner_b130() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Lorehold Skyguard Banner (b130)",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Spirit creatures you control have flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Flying,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyresage (b130) — {4}{R}{W}, 4/4 Spirit Warrior, Haste.
/// Magecraft mints a 2/2 R/W Spirit. A 6-mana finisher that finishes
/// the chain — magecraft mints Spirit, plus the Banner anthem stacks.
pub fn lorehold_pyresage_b130() -> CardDefinition {
    use crate::effect::shortcut::magecraft_mint_token;
    CardDefinition {
        name: "Lorehold Pyresage (b130)",
        cost: cost(&[generic(4), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Reliquarian (b130) — {3}{W}, 2/3 Spirit Cleric. ETB returns
/// target Spirit card with mana value ≤ 3 from your graveyard to your
/// hand. A higher-MV Memorist that recovers larger Spirits (Bell-Ringer
/// MV 3, Sparkmender MV 3, Honorbound MV 2 etc.).
pub fn lorehold_reliquarian_b130() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Reliquarian (b130)",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                    .and(SelectionRequirement::ManaValueAtMost(3)),
            }),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
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
    }
}

/// Lorehold Battle Cantrip (b130) — {2}{R}{W} Instant. Deal 3 damage to
/// target creature, then create a 2/2 R/W Spirit token. The combo of
/// removal + body-on-instant matches Lorehold's "trade for body" style.
pub fn lorehold_battle_cantrip_b130() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battle Cantrip (b130)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                amount: Value::Const(3),
                to: target_filtered(SelectionRequirement::Creature),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyremaster (b130) — {2}{R}, 2/3 Spirit Wizard. Magecraft
/// each opponent loses 1 life — Lorehold's drain-each-opp body on a
/// Spirit-typed Wizard for Spirit-tribal cascades.
pub fn lorehold_pyremaster_b130() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyremaster (b130)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
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
    }
}

// ─── Batch 131: Lorehold synthesised cards ────────────────────────────────────

/// Lorehold Spirit-Warden (b131) — {1}{W}, 1/3 Spirit Cleric, Vigilance.
/// ETB gain 2 life. Compact defensive body for the Spirit/Cleric tribal
/// curve.
pub fn lorehold_spirit_warden_b131() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Warden (b131)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(2)],
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
    }
}

/// Lorehold Pyrosaint (b131) — {1}{R}, 2/1 Spirit Cleric, Haste.
/// Magecraft drains each opponent 1.
pub fn lorehold_pyrosaint_b131() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrosaint (b131)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
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
    }
}

/// Lorehold Relic-Keeper (b131) — {2}{W}, 2/3 Spirit Cleric, Vigilance.
/// ETB exile a target card from any graveyard.
pub fn lorehold_relic_keeper_b131() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Relic-Keeper (b131)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(SelectionRequirement::Any),
            to: ZoneDest::Exile,
        })],
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
    }
}

/// Lorehold Sparkpriest (b131) — {2}{R}{W}, 3/3 Spirit Cleric.
/// Magecraft Seq(DealDamage 1 any target + GainLife 1). Lightning-Helix
/// flavoured magecraft on a body.
pub fn lorehold_sparkpriest_b131() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkpriest (b131)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::DealDamage {
                amount: Value::Const(1),
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
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
    }
}

/// Lorehold Battle-Chant (b131) — {3}{R}{W} Sorcery. Mints 2 Spirit
/// tokens and grants Haste EOT to each Spirit you control.
pub fn lorehold_battle_chant_b131() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battle-Chant (b131)",
        cost: cost(&[generic(3), r(), w()]),
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
                definition: lorehold_spirit_token(),
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Remembrance (b131) — {2}{W} Sorcery. Return target creature
/// card from your graveyard to hand, then scry 1.
pub fn lorehold_remembrance_b131() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Remembrance (b131)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
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
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Ember-Choir (b131) — {1}{R}{W}, 2/2 Spirit Wizard.
/// Magecraft self-pump +1/+0 EOT.
pub fn lorehold_ember_choir_b131() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Choir (b131)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Pyremourner (b131) — {3}{R}, 3/3 Spirit Warrior, Haste.
/// ETB DealDamage 1 to each opponent creature. Hasty mid-curve sweeper.
pub fn lorehold_pyremourner_b131() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Pyremourner (b131)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::DealDamage {
            amount: Value::Const(1),
            to: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
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
    }
}

// ── Batch 132 ───────────────────────────────────────────────────────────────

/// Lorehold Cleric-Recruit (b132) — {W}, 1/2 Spirit Cleric. Vanilla
/// one-drop with the Spirit tribal subtype to feed Lorehold's anthem
/// engines (Spirit Banner, Skyguard Banner, Lectern).
pub fn lorehold_cleric_recruit_b132() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Cleric-Recruit (b132)",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyrescholar (b132) — {1}{R}, 2/2 Spirit Wizard. Magecraft:
/// deal 1 damage to any target. Mirror of `magecraft_ping_any` on a
/// cheap red Spirit body to support the Spirit-tribal Lorehold pool.
pub fn lorehold_pyrescholar_b132() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrescholar (b132)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Spiritforger (b132) — {2}{R}{W}, 2/3 Spirit Wizard.
/// ETB mints a 2/2 Lorehold Spirit token. Mid-curve mint body that
/// pairs with the Lorehold Spirit anthems for tribal pressure.
pub fn lorehold_spiritforger_b132() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritforger (b132)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Ember-Bandit (b132) — {1}{R}, 2/1 Spirit Rogue, Haste.
/// On-attack ping 1 to any target. Cheap aggressive body that doubles
/// as removal-on-curve.
pub fn lorehold_ember_bandit_b132() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Bandit (b132)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_ping_any(1)],
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
    }
}

/// Lorehold Skyforge (b132) — {3}{W} Sorcery. Create 2 1/1 white
/// Spirit tokens with flying (uses SOS's `spirit_token`). Lorehold's
/// Spirit-token go-wide complement to Lorehold Excavation.
pub fn lorehold_skyforge_b132() -> CardDefinition {
    use crate::catalog::spirit_token;
    CardDefinition {
        name: "Lorehold Skyforge (b132)",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Champion's Echo (b132) — {2}{R}{W}, 3/3 Spirit Knight.
/// Vigilance. Whenever this attacks, you gain 1 life. Combines attack
/// trigger gain-life with a vigilant body — solid defensive aggressor.
pub fn lorehold_champions_echo_b132() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Champion's Echo (b132)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_gain_life(1)],
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
    }
}

/// Lorehold Pyresinger (b132) — {2}{R}, 3/2 Spirit Bard. Magecraft:
/// drain 1 to a single opponent (each-opp shape via magecraft_drain).
/// Aggressive red 3-drop with reach via the magecraft drain engine.
pub fn lorehold_pyresinger_b132() -> CardDefinition {
    use crate::effect::shortcut::magecraft_drain;
    CardDefinition {
        name: "Lorehold Pyresinger (b132)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_drain(1)],
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
    }
}

// ── Batch 133 ───────────────────────────────────────────────────────────────

/// Lorehold Spirit-Cleric (b133) — {1}{W}, 2/2 Spirit Cleric, Lifelink.
/// Solid two-drop with intrinsic Lifelink — feeds Lorehold's
/// Spirit + lifegain shells (Light of Promise, anthem decks).
pub fn lorehold_spirit_cleric_b133() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Cleric (b133)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Bell-Ringer II (b133) — {2}{W}, 2/3 Spirit Cleric.
/// ETB mints a 2/2 Lorehold Spirit + gains 2 life. Uses the new
/// `etb_mint_token_and_gain_life` shortcut.
pub fn lorehold_bell_ringer_ii_b133() -> CardDefinition {
    use crate::effect::shortcut::etb_mint_token_and_gain_life;
    CardDefinition {
        name: "Lorehold Bell-Ringer II (b133)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token_and_gain_life(lorehold_spirit_token(), 2)],
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
    }
}

/// Lorehold Sparkstrider (b133) — {3}{R}{R}, 4/3 Spirit Warrior Haste.
/// Magecraft: deal 1 damage to any target. Big hasty body with magecraft
/// reach.
pub fn lorehold_sparkstrider_b133() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkstrider (b133)",
        cost: cost(&[generic(3), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Final Lesson (b132) — {1}{W} Instant. Target creature
/// gets +2/+2 until end of turn and gains Lifelink until end of turn.
/// Single-target combat trick with a defensive twist. Uses the new
/// `pump_and_grant_keyword` shortcut.
pub fn lorehold_final_lesson_b132() -> CardDefinition {
    use crate::effect::shortcut::pump_and_grant_keyword;
    CardDefinition {
        name: "Lorehold Final Lesson (b132)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: pump_and_grant_keyword(2, 2, Keyword::Lifelink),
        activated_abilities: no_abilities(),
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
    }
}

// ── Batch 135 ───────────────────────────────────────────────────────────────

/// Lorehold Skirmisher (b135) — {1}{R} 2/2 Spirit Soldier Haste.
/// Cheap aggressive Spirit 2-drop. Combos with the various
/// magecraft + Spirit-tribal payoffs already in the Lorehold roster.
pub fn lorehold_skirmisher_b135() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skirmisher (b135)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Crackleflame (b135) — {1}{R} Instant. Lorehold Spirit-tribal
/// burn: deal 2 damage to any target, then if you control a Spirit, scry 1.
/// Approximated as DealDamage 2 + Scry 1 (unconditional scry; the Spirit
/// gate is collapsed since the catalog has many Spirit-token creators).
pub fn lorehold_crackleflame_b135() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Crackleflame (b135)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Sparkpilgrim (b135) — {2}{W} 2/3 Spirit Cleric Vigilance.
/// ETB Spirit token. Spirit-tribal mint body with a defender's stat line.
pub fn lorehold_sparkpilgrim_b135() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkpilgrim (b135)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Pyremartyr (b135) — {2}{R} 3/2 Spirit Wizard. When this
/// dies, deal 2 damage to any target. Dies-ping-any payoff at 3 mana —
/// trades into something and brings a 2-damage swing on death.
pub fn lorehold_pyremartyr_b135() -> CardDefinition {
    use crate::effect::shortcut::dies_ping_any;
    CardDefinition {
        name: "Lorehold Pyremartyr (b135)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_ping_any(2)],
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
    }
}

// ── Batch 136 ───────────────────────────────────────────────────────────────

/// Lorehold Ember-Chant (b136) — {2}{R}{W} Sorcery. Creates 2 Lorehold
/// Spirit tokens (2/2 R/W). 4-mana token mint for Spirit tribal.
pub fn lorehold_ember_chant_b136() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Chant (b136)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Skirmisher Captain (b136) — {2}{R} 3/2 Spirit Soldier
/// Haste. On-attack ping-any 1. 3-mana aggressive Spirit body with a
/// rake-on-attack rider.
pub fn lorehold_skirmisher_captain_b136() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skirmisher Captain (b136)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_ping_any(1)],
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
    }
}

/// Lorehold Sage-Choir (b136) — {2}{W} 1/4 Spirit Cleric Vigilance.
/// Magecraft gain 1 life. Defensive Vigilance body with passive
/// lifegain on every spell.
pub fn lorehold_sage_choir_b136() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sage-Choir (b136)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
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
    }
}

/// Lorehold Ember-Sprite (b136) — {R} 1/1 Spirit Elemental Haste.
/// Cheap red one-drop Spirit — feeds Lorehold's Spirit-tribal payoffs.
pub fn lorehold_ember_sprite_b136() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Sprite (b136)",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Elemental],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

// ── Batch 137 ───────────────────────────────────────────────────────────────

/// Lorehold Spirit-Captain (b137) — {3}{R}{W} 3/3 Spirit Soldier Haste.
/// Whenever this creature attacks, create a 2/2 R/W Spirit token.
/// Uses the new `on_attack_create_token` shortcut. Aggressive token
/// engine.
pub fn lorehold_spirit_captain_b137() -> CardDefinition {
    use crate::effect::shortcut::on_attack_create_token;
    CardDefinition {
        name: "Lorehold Spirit-Captain (b137)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_create_token(lorehold_spirit_token())],
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
    }
}

// ── Batch 138 ───────────────────────────────────────────────────────────────

/// Lorehold Pyrocaller (b138) — {1}{R} 2/2 Spirit Shaman. Magecraft
/// 1 damage to any target — Sparkmage-style spellslinger ping body.
pub fn lorehold_pyrocaller_b138() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrocaller (b138)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Spirit-Marshal (b138) — {2}{R}{W} 3/3 Spirit Soldier
/// Vigilance. ETB mint 1 Lorehold Spirit token. 4-mana go-wide body.
pub fn lorehold_spirit_marshal_b138() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Marshal (b138)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Sparkdancer (b138) — {1}{R}{W} 2/2 Spirit Warrior Haste.
/// On-attack ping any 1. Aggressive attack-trigger ping body.
pub fn lorehold_sparkdancer_b138() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkdancer (b138)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_ping_any(1)],
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
    }
}

/// Lorehold Spiritsong (b138) — {2}{R}{W} Sorcery.
/// Seq(CreateToken 1 Lorehold Spirit + GainLife 2). 4-mana token mint
/// + lifegain.
pub fn lorehold_spiritsong_b138() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritsong (b138)",
        cost: cost(&[generic(2), r(), w()]),
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
                definition: lorehold_spirit_token(),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 139 ───────────────────────────────────────────────────────────────

/// Lorehold Pyromancer-Adept (b139) — {2}{R} 2/3 Spirit Shaman.
/// Magecraft 2 damage to target opp creature.
pub fn lorehold_pyromancer_adept_b139() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyromancer-Adept (b139)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
            amount: Value::Const(2),
        })],
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
    }
}

/// Lorehold Spiritwarden (b139) — {3}{R}{W} 4/4 Spirit Soldier
/// Vigilance + Lifelink. Top-end Lorehold finisher.
pub fn lorehold_spiritwarden_b139() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritwarden (b139)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Battle-Witness (b139) — {2}{W} 1/4 Spirit Cleric.
/// On-attack mints a Lorehold Spirit token.
pub fn lorehold_battle_witness_b139() -> CardDefinition {
    use crate::effect::shortcut::on_attack_create_token;
    CardDefinition {
        name: "Lorehold Battle-Witness (b139)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_create_token(lorehold_spirit_token())],
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
    }
}

/// Lorehold Ember-Cleric (b138) — {1}{W} 1/3 Spirit Cleric.
/// Magecraft gain 1 life. Defensive lifegain-on-cast body.
pub fn lorehold_ember_cleric_b138() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Cleric (b138)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
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
    }
}

// ── Batch 141 ───────────────────────────────────────────────────────────────

/// Lorehold Stormcleric (b141) — {2}{R}{W} 3/3 Spirit Cleric Haste.
/// ETB mint Spirit token. 4-mana go-wide Spirit-tribal payoff with haste
/// to enable an immediate attack with the newly minted token.
pub fn lorehold_stormcleric_b141() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Stormcleric (b141)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Pyrosage (b141) — {1}{R} 2/1 Spirit Shaman.
/// Magecraft ping-each-opp 1.
pub fn lorehold_pyrosage_b141() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_each_opp;
    CardDefinition {
        name: "Lorehold Pyrosage (b141)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_each_opp(1)],
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
    }
}

/// Lorehold Spiritforge (b141) — {3}{R}{W} Sorcery. Create two Spirit
/// tokens + you gain 2 life. Lorehold go-wide token engine.
pub fn lorehold_spiritforge_b141() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritforge (b141)",
        cost: cost(&[generic(3), r(), w()]),
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
                definition: lorehold_spirit_token(),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Ember-Soldier (b141) — {2}{R} 3/2 Spirit Soldier Haste.
/// Attack trigger ping 1 to target opp creature. Aggressive 3-drop with
/// repeated burn-creature removal on attacks.
pub fn lorehold_ember_soldier_b141() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Soldier (b141)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Sparkscholar III (b141) — {2}{R}{W} 2/2 Spirit Wizard.
/// Magecraft mint a 2/2 R/W Spirit token. Uses the new
/// `magecraft_mint_spirit()` helper. Lorehold spell-engine that grows
/// the Spirit board on every IS cast.
pub fn lorehold_sparkscholar_iii_b141() -> CardDefinition {
    use crate::effect::shortcut::magecraft_mint_spirit;
    CardDefinition {
        name: "Lorehold Sparkscholar III (b141)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_mint_spirit()],
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
    }
}

// ── Batch 142 ───────────────────────────────────────────────────────────────

/// Lorehold Pyroscribe (b142) — {2}{R} 3/2 Human Wizard. Magecraft
/// deal 1 damage to each opponent creature. Spellslinger boardwipe-on-
/// drip.
pub fn lorehold_pyroscribe_b142() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyroscribe (b142)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
            amount: Value::Const(1),
        })],
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
    }
}

/// Lorehold Spiritbond (b142) — {W}{R} Instant. Target creature you
/// control gets +2/+1 EOT and gains Haste EOT. 2-mana combat trick
/// with haste rider for an end-step alpha strike.
pub fn lorehold_spiritbond_b142() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritbond (b142)",
        cost: cost(&[w(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Stoneveil (b142) — {2}{W} 1/4 Human Cleric Vigilance.
/// ETB reanimate a creature card with mana value ≤ 2 from your
/// graveyard. 3-mana low-curve recursion body.
pub fn lorehold_stoneveil_b142() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Stoneveil (b142)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMost(2)),
            }),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        })],
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
    }
}

/// Lorehold Spiritmender (b142) — {3}{R}{W} 3/3 Spirit Cleric Flying.
/// ETB Seq(GainLife 4 + mint Spirit token). 5-mana race breaker top-end.
pub fn lorehold_spiritmender_b142() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritmender (b142)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(4),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
            },
        ]))],
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
    }
}

/// Lorehold Spellfire (b142) — {2}{R} Sorcery. Deal 4 damage to any
/// target. 3-mana burn finisher (Searing Blood-class).
pub fn lorehold_spellfire_b142() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spellfire (b142)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(4),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 143 ───────────────────────────────────────────────────────────────

/// Lorehold Ember-Acolyte (b143) — {R}{W} 2/2 Human Cleric. Magecraft
/// Seq(GainLife 1 + DealDamage 1 to any target). Apprentice template.
pub fn lorehold_ember_acolyte_b143() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Acolyte (b143)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            },
        ]))],
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
    }
}

/// Lorehold Pyromancer (b143) — {1}{R} 2/1 Human Wizard. Magecraft deal
/// 2 damage to target opponent.
pub fn lorehold_pyromancer_b143() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyromancer (b143)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(2),
        })],
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
    }
}

/// Lorehold Stonemason (b143) — {2}{R}{W} 3/3 Spirit Cleric. ETB returns
/// target creature card from your gy to hand.
pub fn lorehold_stonemason_b143() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Stonemason (b143)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature,
            }),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
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
    }
}

/// Lorehold Inferno (b143) — {3}{R} Sorcery. Deal 5 damage to target creature
/// or planeswalker. 4-mana big removal.
pub fn lorehold_inferno_b143() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Inferno (b143)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(5),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spirit-Bond (b143) — {2}{W} 2/3 Spirit Cleric Flying.
/// "Whenever another Spirit enters the battlefield under your control,
/// put a +1/+1 counter on this creature." Spirit-tribal scaler.
pub fn lorehold_spirit_bond_b143() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Bond (b143)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(crate::card::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Spirit),
                }),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Flamekeeper (b143) — {2}{R} 3/2 Spirit Soldier Haste.
/// 3-mana aggressive haste body — combat finisher.
pub fn lorehold_flamekeeper_b143() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Flamekeeper (b143)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Battle-Chant (b143) — {R}{W} Instant. Target creature you
/// control gets +2/+2 EOT and Trample EOT. 2-mana big combat trick.
pub fn lorehold_battle_chant_b143() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battle-Chant (b143)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 144 ───────────────────────────────────────────────────────────────

/// Lorehold Ignis (b144) — {2}{R} Sorcery. Deal 3 damage divided among
/// any number of targets — collapsed to "3 damage to target creature
/// or player" (split-damage primitive ⏳).
pub fn lorehold_ignis_b144() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ignis (b144)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(3),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Conjurer (b144) — {2}{R}{W} 2/4 Spirit Cleric Vigilance.
/// "Whenever you cast an IS spell, create a 2/2 R/W Spirit token."
/// Stax-style synergy with Magecraft shells.
pub fn lorehold_conjurer_b144() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Conjurer (b144)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::magecraft_mint_spirit()],
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
    }
}

/// Lorehold Pyroflame (b144) — {R}{W} Instant. Deal 2 damage and gain
/// 2 life. Apprentice-style template at instant speed.
pub fn lorehold_pyroflame_b144() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyroflame (b144)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 145 ───────────────────────────────────────────────────────────────

/// Lorehold Spiritcaller (b145) — {3}{R}{W} 3/3 Spirit Wizard. ETB
/// returns target Spirit card from your gy → bf.
pub fn lorehold_spiritcaller_b145() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Lorehold Spiritcaller (b145)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::HasCreatureType(CreatureType::Spirit),
            }),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        })],
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
    }
}

/// Lorehold Inferno-Acolyte (b145) — {R} 1/1 Human Cleric.
/// Magecraft drain 1 (each opp loses 1; you gain 1). 1-drop apprentice.
pub fn lorehold_inferno_acolyte_b145() -> CardDefinition {
    use crate::effect::shortcut::magecraft_drain;
    CardDefinition {
        name: "Lorehold Inferno-Acolyte (b145)",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_drain(1)],
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
    }
}

/// Lorehold Knight-Errant (b145) — {2}{W} 2/3 Spirit Soldier
/// Vigilance + First Strike. 3-mana defender + combat presence.
pub fn lorehold_knight_errant_b145() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Knight-Errant (b145)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Embermage (b144) — {1}{R} 2/2 Human Wizard. Cycling {2}.
pub fn lorehold_embermage_b144() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Embermage (b144)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Cinderscholar (b143) — {1}{R}{W} 2/3 Spirit Wizard. ETB
/// gain 2 life + magecraft deal 1 damage to any target.
pub fn lorehold_cinderscholar_b143() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Cinderscholar (b143)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_gain_life(2),
            magecraft_ping_any(1),
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
    }
}


// ── Batch 146 ───────────────────────────────────────────────────────────────

/// Lorehold Echocaller (b146) — {1}{R}{W} 2/3 Spirit Wizard. ETB returns
/// target IS card from your gy → hand. 3-mana Pillardrop mini.
pub fn lorehold_echocaller_b146() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Echocaller (b146)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            }),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
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
    }
}

/// Lorehold Spirit-Glyph (b146) — {R}{W} Sorcery. Mints a 2/2 R/W Spirit
/// token (the shared `lorehold_spirit_token()` helper). 2-mana cheap
/// token mint at sorcery speed.
pub fn lorehold_spirit_glyph_b146() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Glyph (b146)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Ember-Adept (b146) — {1}{R} 2/2 Spirit Wizard. Magecraft
/// 1 damage to any target. Same shape as Lorehold Sparkflinger.
pub fn lorehold_ember_adept_b146() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Adept (b146)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Pyresinger (b146) — {3}{R}{W} 3/3 Spirit Cleric Haste.
/// ETB mints 2 Spirit tokens (the shared `lorehold_spirit_token()`).
/// 5-mana hasty go-wide finisher.
pub fn lorehold_pyresinger_b146() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyresinger (b146)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 2)],
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
    }
}

/// Lorehold Spirit-Burst (b146) — {2}{R} Instant. Deals 3 damage to
/// any target. 3-mana Shock-on-a-bigger-cost — flexible removal at
/// instant tempo.
pub fn lorehold_spirit_burst_b146() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Burst (b146)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(3),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Soulrender (b146) — {2}{W} 2/4 Spirit Soldier Vigilance +
/// Lifelink. 3-mana defensive double-keyword body. Stacks with
/// Lorehold Anthemist for tribal-pump value.
pub fn lorehold_soulrender_b146() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Soulrender (b146)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Battle-Sage (b146) — {2}{R}{W} 3/3 Spirit Cleric.
/// Magecraft +1/+1 counter on target friendly Spirit. Spirit-tribal
/// growth payoff.
pub fn lorehold_battle_sage_b146() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battle-Sage (b146)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyrebound (b146) — {R} 1/1 Spirit Warrior Haste.
/// Aggressive 1-drop Spirit hasher — feeds Lorehold Anthemist's
/// pump and Quintorius's tribal value.
pub fn lorehold_pyrebound_b146() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrebound (b146)",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Spirit-Decree (b146) — {3}{R}{W} Sorcery. Seq(DealDamage 1
/// to each opp creature + CreateToken 1 Spirit). 5-mana sweep + body.
pub fn lorehold_spirit_decree_b146() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Decree (b146)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                amount: Value::Const(1),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Glyph-Strike (b146) — {1}{R} Instant. Deals 2 damage to
/// target creature. Magecraft cantrip-style burn — feeds Lorehold
/// Apprentice's drain trigger.
pub fn lorehold_glyph_strike_b146() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Glyph-Strike (b146)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 147 ───────────────────────────────────────────────────────────────

/// Lorehold Glyphcaster (b147) — {2}{R}{W} 3/3 Spirit Wizard. Magecraft
/// 1 damage to any target. Same shape as Lorehold Ember-Adept but +1 mana
/// for +1/+1 stats.
pub fn lorehold_glyphcaster_b147() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Glyphcaster (b147)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Ironwarden (b147) — {3}{W} 2/5 Spirit Soldier Vigilance.
/// 4-mana wall — defensive vigilance body.
pub fn lorehold_ironwarden_b147() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ironwarden (b147)",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Pyrehowler (b147) — {3}{R}{W} Sorcery. DealDamage 5 to any
/// target. 5-mana burn finisher.
pub fn lorehold_pyrehowler_b147() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrehowler (b147)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(5),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Cinderscry (b147) — {R} Sorcery. DealDamage 1 to any target +
/// Scry 1. 1-mana cantrip-burn.
pub fn lorehold_cinderscry_b147() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Cinderscry (b147)",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Spirit Banner-Bearer (b147) — {2}{W} 2/3 Spirit Soldier. Static "Other
/// Spirit creatures you control get +1/+0." Mirror of Inkling Banner-Bearer.
pub fn spirit_banner_bearer_b147() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Spirit Banner-Bearer (b147)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Spirit creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 148 ───────────────────────────────────────────────────────────────

/// Lorehold Lightcaller (b148) — {2}{R}{W} 3/3 Spirit Cleric Lifelink.
/// ETB DealDamage 2 to any target. 4-mana double-keyword aggro engine.
pub fn lorehold_lightcaller_b148() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Lightcaller (b148)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(2),
        })],
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
    }
}

/// Lorehold Ember-Wraith (b148) — {3}{R} 4/2 Spirit Wizard Haste.
/// Magecraft Treasure. 4-mana ramp/burn body.
pub fn lorehold_ember_wraith_b148() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Wraith (b148)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![{
            use crate::effect::shortcut::magecraft_treasure;
            magecraft_treasure()
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
    }
}

/// Lorehold Cinderlist (b148) — {1}{R} Sorcery. DealDamage 2 to target
/// creature OR player. 2-mana flexible burn.
pub fn lorehold_cinderlist_b148() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Cinderlist (b148)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Skystalker (b148) — {3}{W} 2/4 Spirit Soldier Flying +
/// Vigilance. 4-mana defensive flier.
pub fn lorehold_skystalker_b148() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skystalker (b148)",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Spirit-Smith (b148) — {2}{R}{W} 3/3 Spirit Warrior. ETB
/// mints 1 Spirit token + grants Haste EOT to it (via the layer-6
/// keyword-grant path). 4-mana double-tempo body.
pub fn lorehold_spirit_smith_b148() -> CardDefinition {
    use crate::effect::shortcut::create_token_with_keyword;
    CardDefinition {
        name: "Lorehold Spirit-Smith (b148)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(create_token_with_keyword(
            PlayerRef::You,
            1,
            lorehold_spirit_token(),
            Keyword::Haste,
            Duration::EndOfTurn,
        ))],
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
    }
}

// ── Batch 149 ───────────────────────────────────────────────────────────────

/// Lorehold Echobreaker (b149) — {1}{R}{W} 2/2 Spirit Soldier Persist.
/// Recursion-friendly Spirit body — on dying without a -1/-1 counter,
/// returns with a -1/-1 counter on it (CR 702.79).
pub fn lorehold_echobreaker_b149() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Echobreaker (b149)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Persist],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Eternal-Phoenix (b149) — {2}{R}{W} 2/2 Phoenix Spirit
/// Flying + Haste + Undying. Recurring evasive haster.
pub fn lorehold_eternal_phoenix_b149() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Eternal-Phoenix (b149)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phoenix, CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::Undying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Pyre-Stalker (b149) — {3}{R} 4/3 Spirit Warrior Trample.
/// 4-mana big trampler.
pub fn lorehold_pyre_stalker_b149() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyre-Stalker (b149)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 150 ───────────────────────────────────────────────────────────────

/// Lorehold Embermage (b150) — {1}{R}{W} 2/3 Spirit Wizard. Magecraft
/// 1 damage to target opponent.
pub fn lorehold_embermage_b150() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Embermage (b150)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
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
    }
}

/// Lorehold Spiritforge (b150) — {2}{R}{W} 3/3 Spirit Warrior. ETB
/// create a 2/2 R/W Spirit token (Lorehold spirit).
pub fn lorehold_spiritforge_b150() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritforge (b150)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Ancestor (b150) — {3}{R}{W} 4/5 Spirit Soldier Vigilance.
/// Big mid-curve defender Spirit.
pub fn lorehold_ancestor_b150() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ancestor (b150)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Sparkmage (b150) — {R} 1/1 Human Wizard. Magecraft ping any
/// for 1.
pub fn lorehold_sparkmage_b150() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkmage (b150)",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Bonfire (b150) — {2}{R} Sorcery. Deal 4 damage to target
/// creature, 1 damage to its controller. Two-stage red removal.
pub fn lorehold_bonfire_b150() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::Selector as Sel;
    CardDefinition {
        name: "Lorehold Bonfire (b150)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(4),
            },
            Effect::DealDamage {
                to: Sel::Player(PlayerRef::ControllerOf(Box::new(Sel::Target(0)))),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spirit-Tender (b150) — {1}{W} 1/3 Spirit Cleric. On-attack
/// gain 1 life.
pub fn lorehold_spirit_tender_b150() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Tender (b150)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_gain_life(1)],
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
    }
}

/// Lorehold Ember Strike (b150) — {R} Instant. Deal 2 damage to target
/// creature or planeswalker. Classic burn-removal.
pub fn lorehold_ember_strike_b150() -> CardDefinition {
    use crate::card::SelectionRequirement;
    CardDefinition {
        name: "Lorehold Ember Strike (b150)",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 151 ───────────────────────────────────────────────────────────────

/// Lorehold Skirmisher (b151) — {1}{R} 2/2 Spirit Soldier Haste.
/// Aggressive 2-mana attacker.
pub fn lorehold_skirmisher_b151() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skirmisher (b151)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Pyrelore (b151) — {3}{R}{W} Sorcery. Deal 4 damage to
/// target creature you don't control + you gain 4 life.
pub fn lorehold_pyrelore_b151() -> CardDefinition {
    use crate::card::SelectionRequirement;
    CardDefinition {
        name: "Lorehold Pyrelore (b151)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                amount: Value::Const(4),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(4),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spirit-Guide (b151) — {2}{R} 2/2 Spirit. Magecraft mint
/// a 2/2 R/W Spirit token.
pub fn lorehold_spirit_guide_b151() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Guide (b151)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::magecraft_mint_spirit()],
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
    }
}

/// Lorehold Battlemage (b151) — {2}{W} 2/2 Spirit Wizard. ETB Vigilance
/// to target creature you control until EOT.
pub fn lorehold_battlemage_b151() -> CardDefinition {
    use crate::card::SelectionRequirement;
    CardDefinition {
        name: "Lorehold Battlemage (b151)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            keyword: Keyword::Vigilance,
            duration: Duration::EndOfTurn,
        })],
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
    }
}

/// Lorehold Sun-Spirit (b151) — {3}{W} 3/3 Spirit Cleric Flying.
/// Solid evasive 4-mana flier.
pub fn lorehold_sun_spirit_b151() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sun-Spirit (b151)",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

// ── Batch 152 ───────────────────────────────────────────────────────────────

/// Lorehold Spirit-Stalker (b152) — {1}{R} 2/2 Spirit Soldier Haste +
/// Menace. Aggressive 2-mana attacker.
pub fn lorehold_spirit_stalker_b152() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Stalker (b152)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste, Keyword::Menace],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Ember-Cleric (b152) — {1}{W} 1/3 Spirit Cleric.
/// ETB gain 2 life + scry 1.
pub fn lorehold_ember_cleric_b152() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Cleric (b152)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]))],
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
    }
}

/// Lorehold Pyre-Ancient (b152) — {4}{R}{W} 5/5 Spirit Giant Vigilance +
/// Trample. Big finisher.
pub fn lorehold_pyre_ancient_b152() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyre-Ancient (b152)",
        cost: cost(&[generic(4), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Giant],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Pyromancer (b152) — {2}{R} 2/2 Human Wizard. Magecraft 1
/// damage to target creature/player.
pub fn lorehold_pyromancer_b152() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyromancer (b152)",
        cost: cost(&[generic(2), r()]),
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
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

// ── Batch 154 ───────────────────────────────────────────────────────────────

/// Lorehold Spirit-Surger (b154) — {2}{R}{W} 3/3 Spirit Cleric.
/// Attacks/SelfSource → mint a 2/2 R/W Spirit token via the new
/// `on_attack_mint_lorehold_spirit()` shortcut. Per-attack token
/// engine on a beefy body.
pub fn lorehold_spirit_surger_b154() -> CardDefinition {
    use crate::effect::shortcut::on_attack_mint_lorehold_spirit;
    CardDefinition {
        name: "Lorehold Spirit-Surger (b154)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_mint_lorehold_spirit()],
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
    }
}

/// Lorehold Reflux (b154) — {R}{W} Instant. Seq(DealDamage 2 any
/// target + GainLife 2). 2-mana micro-Lightning-Helix.
pub fn lorehold_reflux_b154() -> CardDefinition {
    use crate::effect::shortcut::{gain_life, deal};
    CardDefinition {
        name: "Lorehold Reflux (b154)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            deal(2, target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            )),
            gain_life(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Battlespirit (b154) — {3}{R}{W} 3/3 Spirit Soldier
/// First Strike + Vigilance. ETB mints a 2/2 R/W Spirit token.
/// Aggressive token + finisher body.
pub fn lorehold_battlespirit_b154() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battlespirit (b154)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Cinderspeaker (b154) — {1}{R}{W} 2/2 Spirit Wizard.
/// Magecraft ping target Creature/Player/Planeswalker for 1. Compact
/// Lorehold magecraft engine.
pub fn lorehold_cinderspeaker_b154() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Cinderspeaker (b154)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Smiterite (b154) — {2}{R} 3/2 Spirit Shaman, Haste.
/// Magecraft +1/+0 self-pump EOT. Aggressive spell-slinger build-
/// around — pairs with Galvanic Iteration / copy-spell triggers
/// for stacked pumps.
pub fn lorehold_smiterite_b154() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Smiterite (b154)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Memoryflame (b154) — {2}{R}{W} Sorcery. Seq(Deal 3 dmg
/// any target + Move target IS card from gy → hand). 4-mana burn +
/// graveyard recursion combo.
pub fn lorehold_memoryflame_b154() -> CardDefinition {
    use crate::effect::shortcut::deal;
    CardDefinition {
        name: "Lorehold Memoryflame (b154)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            deal(3, target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            )),
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                }),
                to: ZoneDest::Hand(PlayerRef::You),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spirit-Banner (b154) — {2}{R}{W} 2/2 Spirit. Static:
/// Other Spirit creatures you control get +1/+0. Spirit-tribal
/// anthem complement to Quintorius / Tenured Inkcaster shapes.
pub fn lorehold_spirit_banner_b154() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Lorehold Spirit-Banner (b154)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Spirit creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Stratagem (b154) — {3}{R}{W} Sorcery. Seq(CreateToken(2
/// lorehold_spirit_token) + DealDamage 3 to opp). 5-mana 2 bodies +
/// 3 dmg to opp burst — Lorehold finisher slot.
pub fn lorehold_stratagem_b154() -> CardDefinition {
    use crate::effect::shortcut::deal;
    CardDefinition {
        name: "Lorehold Stratagem (b154)",
        cost: cost(&[generic(3), r(), w()]),
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
                definition: lorehold_spirit_token(),
            },
            deal(3, Selector::Player(PlayerRef::EachOpponent)),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Searingscholar (b154) — {2}{R} 2/3 Spirit Wizard.
/// Magecraft Drain each opp 1 — Lorehold drain template at the 3-mana
/// slot.
pub fn lorehold_searingscholar_b154() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Searingscholar (b154)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
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
    }
}

/// Lorehold Cinderward (b154) — {2}{W} 2/4 Spirit Cleric Vigilance.
/// ETB gain 3 life. Defensive lifegain anchor.
pub fn lorehold_cinderward_b154() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Cinderward (b154)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(3)],
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
    }
}

/// Lorehold Strikeritual (b154) — {1}{R}{W} Sorcery. Seq(DealDamage
/// 2 any target + CreateToken 1 Spirit). 3-mana burn + body.
pub fn lorehold_strikeritual_b154() -> CardDefinition {
    use crate::effect::shortcut::deal;
    CardDefinition {
        name: "Lorehold Strikeritual (b154)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            deal(2, target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            )),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 155 (modern_decks) — 8 new Lorehold cards ────────────────────────

/// Lorehold Chronicler (b155) — {R}{W} 2/2 Human Cleric. Magecraft
/// pings any target for 1. Aggressive 2-drop spell-pinger.
pub fn lorehold_chronicler_b155() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Chronicler (b155)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Spirit Crusader II (b155) — {2}{R}{W} 3/3 Spirit Cleric Flying.
/// ETB: deal 2 damage to any target. Pillardrop Skyguide-style
/// flying body + Lightning Strike rider.
pub fn spirit_crusader_ii_b155() -> CardDefinition {
    use crate::effect::shortcut::{deal, etb};
    CardDefinition {
        name: "Spirit Crusader II (b155)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(deal(2, target_filtered(
            SelectionRequirement::Creature
                .or(SelectionRequirement::Player)
                .or(SelectionRequirement::Planeswalker),
        )))],
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
    }
}

/// Lorehold Reverent (b155) — {R}{W} 2/2 Human Cleric. ETB: gain
/// 2 life. Lifegain on entry — composes with Blech / Pest Mascot.
pub fn lorehold_reverent_b155() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Reverent (b155)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(2)],
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
    }
}

/// Lorehold Ironscribe (b155) — {2}{R}{W} 3/3 Dwarf Cleric.
/// On-attack: mint a 2/2 R/W Spirit token. Lorehold token-snowball
/// midrange.
pub fn lorehold_ironscribe_b155() -> CardDefinition {
    use crate::effect::shortcut::on_attack_create_token;
    CardDefinition {
        name: "Lorehold Ironscribe (b155)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_create_token(lorehold_spirit_token())],
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
    }
}

/// Pillardrop Veteran (b155) — {3}{R}{W} 3/4 Spirit Cleric Flying.
/// Vanilla flying body. Lorehold finisher curve.
pub fn pillardrop_veteran_b155() -> CardDefinition {
    CardDefinition {
        name: "Pillardrop Veteran (b155)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Flamebrand (b155) — {1}{R} Instant. Seq(DealDamage 3
/// to target creature + GainLife 1). Lightning-Helix-lite — 2 mana,
/// less damage and lifegain.
pub fn lorehold_flamebrand_b155() -> CardDefinition {
    use crate::effect::shortcut::{deal, gain_life};
    CardDefinition {
        name: "Lorehold Flamebrand (b155)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            deal(3, target_filtered(SelectionRequirement::Creature)),
            gain_life(1),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spirit-Captain (b155) — {3}{R}{W} 4/4 Spirit Soldier.
/// Vigilance + on-attack 1 damage to each opponent. Lorehold
/// attacking finisher.
pub fn lorehold_spirit_captain_b155() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Lorehold Spirit-Captain (b155)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
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
    }
}

/// Lorehold Pyromancer (b155) — {R}{W} 1/2 Human Wizard. Magecraft:
/// ping target for 1 + gain 1 life. Combines Lorehold lifegain +
/// damage on every spell.
pub fn lorehold_pyromancer_b155() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyromancer (b155)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
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
    }
}

// ── Batch 156 (modern_decks) — Lorehold attack-anchor cards ────────────────
//
// These cards exercise the new batch-fanout fix in the trigger dispatcher
// (push c4b7b14): each attacker in a multi-attacker batch now correctly
// fires "Another of yours attacks" triggers once per attacker.

/// Lorehold Banner (b156) — {2}{R}{W} Enchantment. Whenever another
/// creature you control attacks, that creature gets +1/+0 until end of
/// turn. Multi-attacker pump anchor — exercises the per-attacker
/// broadcast.
pub fn lorehold_banner_b156() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Banner (b156)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnotherOfYours),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(1),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Marshal (b156) — {3}{R}{W} 3/3 Spirit Cleric. Whenever
/// another creature you control attacks, you gain 1 life. Lifegain
/// fan-out per attacker.
pub fn lorehold_marshal_b156() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Marshal (b156)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnotherOfYours),
            effect: Effect::GainLife {
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Banner-Bearer (b156) — {1}{R}{W} 1/3 Human Soldier. When
/// another creature you control attacks, that creature gains haste
/// until end of turn. Battlefield-wide haste enabler.
pub fn lorehold_banner_bearer_b156() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Banner-Bearer (b156)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnotherOfYours),
            effect: Effect::GrantKeyword {
                what: Selector::TriggerSource,
                keyword: Keyword::Haste,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── batch 155 — Lorehold ───────────────────────────────────────────────────

/// Lorehold Glyphbearer (b155) — {1}{R} 2/2 Human Soldier. Magecraft
/// pings any target for 1.
pub fn lorehold_glyphbearer_b155() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Lorehold Glyphbearer (b155)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Watchspirit (b155) — {2}{W} 2/3 Spirit Cleric Flying.
/// ETB gain 2 life.
pub fn lorehold_watchspirit_b155() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Watchspirit (b155)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(2)],
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
    }
}

/// Lorehold Spiritforge (b155) — {2}{R}{W} 3/3 Human Cleric.
/// Attack trigger mints a 2/2 R/W Spirit token (Lorehold spirit). Tempo
/// + tribal engine.
pub fn lorehold_spiritforge_b155() -> CardDefinition {
    use crate::effect::shortcut::on_attack_mint_lorehold_spirit;
    CardDefinition {
        name: "Lorehold Spiritforge (b155)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_mint_lorehold_spirit()],
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
    }
}

/// Lorehold Pyrescholar (b155) — {1}{R} 2/1 Human Wizard. ETB deals 1
/// damage to any target.
pub fn lorehold_pyrescholar_b155() -> CardDefinition {
    use crate::effect::shortcut::deal;
    CardDefinition {
        name: "Lorehold Pyrescholar (b155)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(deal(1, target_filtered(
            SelectionRequirement::Creature
                .or(SelectionRequirement::Player)
                .or(SelectionRequirement::Planeswalker),
        )))],
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
    }
}

/// Lorehold Bonewright (b155) — {R}{W} 2/2 Human Cleric Lifelink.
/// 2-mana lifelink body — Lorehold's "drains while attacking" curve.
pub fn lorehold_bonewright_b155() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Bonewright (b155)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Skystrider (b155) — {2}{R}{W} 3/3 Spirit Soldier Flying +
/// Vigilance. Premium evasive Lorehold defender.
pub fn lorehold_skystrider_b155() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skystrider (b155)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Battlechant (b155) — {1}{R}{W} Sorcery. Seq(DealDamage 2
/// any target + GainLife 2). 3-mana balanced damage + life.
pub fn lorehold_battlechant_b155() -> CardDefinition {
    use crate::effect::shortcut::deal;
    CardDefinition {
        name: "Lorehold Battlechant (b155)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            deal(2, target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            )),
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Ancestralist (b155) — {3}{R}{W} 3/4 Human Cleric.
/// ETB returns target creature card from your gy to hand. 5-mana
/// value body for grindy Lorehold midrange.
pub fn lorehold_ancestralist_b155() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ancestralist (b155)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature,
            }),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
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
    }
}

/// Lorehold Echocaller (b155) — {1}{W} 1/3 Human Cleric.
/// Magecraft GainLife 1.
pub fn lorehold_echocaller_b155() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Echocaller (b155)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
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
    }
}

/// Lorehold Pyrebolt (b155) — {R} Instant. DealDamage 2 to target
/// creature. 1-mana cheap burn — Lorehold's red splash removal.
pub fn lorehold_pyrebolt_b155() -> CardDefinition {
    use crate::effect::shortcut::deal;
    CardDefinition {
        name: "Lorehold Pyrebolt (b155)",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: deal(2, target_filtered(SelectionRequirement::Creature)),
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Vanguard (b155) — {2}{W} 2/4 Human Soldier First Strike.
/// Resilient anchor with first strike — Lorehold's go-wide protector.
pub fn lorehold_vanguard_b155() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Vanguard (b155)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Spirit Caller (b155) — {3}{R}{W} 3/3 Human Cleric.
/// ETB mints 2 R/W Spirit tokens. 5-mana spirit-tribal lord.
pub fn lorehold_spirit_caller_b155() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit Caller (b155)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: lorehold_spirit_token(),
        })],
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
    }
}

// ── Batch 158 (modern_decks) — Lorehold cards ──────────────────────────────

/// Lorehold Wallscribe (b158) — {1}{W} 1/3 Spirit Cleric Vigilance.
/// ETB gain 1 life. Defensive smoother.
pub fn lorehold_wallscribe_b158() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Wallscribe (b158)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(1)],
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
    }
}

/// Lorehold Embermage (b158) — {2}{R} 2/2 Spirit Wizard.
/// Magecraft 1 damage to any target. 3-mana magecraft pinger.
pub fn lorehold_embermage_b158() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Embermage (b158)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Spirit-Drummer (b158) — {1}{R}{W} 2/2 Spirit Warrior Haste.
/// Magecraft self-pump +1/+0 EOT. Aggressive magecraft scaling.
pub fn lorehold_spirit_drummer_b158() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Drummer (b158)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Stoneflame (b158) — {1}{R} Sorcery.
/// Deal 3 damage to target creature. Cheap red removal.
pub fn lorehold_stoneflame_b158() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Stoneflame (b158)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(3),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Reaver (b158) — {2}{R} 3/2 Spirit Warrior Haste.
/// Aggressive 3-mana red hasty Spirit.
pub fn lorehold_reaver_b158() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Reaver (b158)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Spirit Vanguard II (b158) — {1}{W} 1/2 Spirit Cleric Vigilance.
/// Vanilla Spirit-tribal anthem fodder.
pub fn spirit_vanguard_ii_b158() -> CardDefinition {
    CardDefinition {
        name: "Spirit Vanguard II (b158)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Spectermage (b158) — {R}{W} 2/2 Spirit Wizard.
/// Magecraft 1 damage to any target.
pub fn lorehold_spectermage_b158() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectermage (b158)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Spirit-Caster (b158) — {2}{R}{W} 2/2 Spirit Cleric.
/// ETB mints 1 R/W Spirit token. Spirit-tribal value body.
pub fn lorehold_spirit_caster_b158() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Caster (b158)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(lorehold_spirit_token(), 1)],
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
    }
}

/// Lorehold Crusader (b158) — {3}{R}{W} 4/3 Spirit Knight First Strike.
/// 5-mana first-strike body.
pub fn lorehold_crusader_b158() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Crusader (b158)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Spellsong (b158) — {R}{W} Instant.
/// Deal 2 damage to any target; gain 2 life. Lightning-Helix template.
pub fn lorehold_spellsong_b158() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spellsong (b158)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spiritforger (b158) — {2}{R}{W} 3/3 Spirit Cleric Vigilance.
/// Spirit-tribal anthem: Other Spirits you control get +1/+0.
pub fn lorehold_spiritforger_b158() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Lorehold Spiritforger (b158)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Spirit creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Stonewright (b158) — {R}{W} Sorcery.
/// Mints 2 R/W Spirit tokens — cheap go-wide spirit minter.
pub fn lorehold_stonewright_b158() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Stonewright (b158)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyremage (b158) — {R}{W} 2/1 Spirit Wizard Haste.
/// Aggressive haste body — Lorehold racer.
pub fn lorehold_pyremage_b158() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyremage (b158)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Spectral-Lance (b158) — {2}{R}{W} Sorcery.
/// Seq(DealDamage 3 + CreateToken 1 Spirit). 4-mana burn + body.
pub fn lorehold_spectral_lance_b158() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectral-Lance (b158)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Recallmage (b158) — {2}{R} 3/2 Spirit Wizard.
/// ETB return target creature card from your graveyard to your hand.
pub fn lorehold_recallmage_b158() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Recallmage (b158)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature,
            }),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
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
    }
}

/// Lorehold Spectral-Watcher (b158) — {1}{W} 1/3 Spirit Cleric Vigilance.
/// ETB Scry 1.
pub fn lorehold_spectral_watcher_b158() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Lorehold Spectral-Watcher (b158)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_scry(1)],
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
    }
}

// ── Batch 159 (modern_decks) — More Lorehold cards ─────────────────────────

/// Lorehold Pyrescholar (b159) — {2}{R} 2/3 Spirit Wizard.
/// Magecraft 1 damage to each opp.
pub fn lorehold_pyrescholar_b159() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_each_opp;
    CardDefinition {
        name: "Lorehold Pyrescholar (b159)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_each_opp(1)],
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
    }
}

/// Lorehold Sentinel (b159) — {3}{W} 2/4 Spirit Soldier Vigilance.
/// 4-mana defensive vigilance body.
pub fn lorehold_sentinel_b159() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sentinel (b159)",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Ember-Mage (b159) — {R} 1/1 Spirit Wizard.
/// Magecraft 1 damage to any target. Cheapest Lorehold ping engine.
pub fn lorehold_ember_mage_b159() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ember-Mage (b159)",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Spectral Cavalry (b159) — {2}{R}{W} 3/3 Spirit Knight Haste.
/// 4-mana hasty mid-range Spirit.
pub fn lorehold_spectral_cavalry_b159() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectral Cavalry (b159)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Battlescroll (b159) — {3}{R}{W} Sorcery.
/// Mint 2 Spirit tokens with haste EOT.
pub fn lorehold_battlescroll_b159() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::card::ActivatedAbility;
    let _ = TokenDefinition {
        name: "Spirit".to_string(),
        power: 2,
        toughness: 2,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        supertypes: vec![],
        subtypes: Subtypes::default(),
        activated_abilities: vec![] as Vec<ActivatedAbility>,
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Lorehold Battlescroll (b159)",
        cost: cost(&[generic(3), r(), w()]),
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
                definition: lorehold_spirit_token(),
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 160 (modern_decks) — Lorehold additions ──────────────────────────

/// Lorehold Spectralguard (b160) — {2}{W} 1/4 Spirit Cleric Vigilance.
/// Defensive vigilance body.
pub fn lorehold_spectralguard_b160() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectralguard (b160)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Sparkpriest (b160) — {1}{R}{W} 2/3 Spirit Cleric.
/// Magecraft each opp loses 1 life.
pub fn lorehold_sparkpriest_b160() -> CardDefinition {
    use crate::effect::shortcut::magecraft_drain_each_opp;
    CardDefinition {
        name: "Lorehold Sparkpriest (b160)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
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
    }
}

/// Lorehold Bonewright (b160) — {2}{R}{W} 3/3 Spirit Warrior First Strike.
/// 4-mana first strike aggressor.
pub fn lorehold_bonewright_b160() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Bonewright (b160)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Recallsmith (b160) — {3}{R}{W} 3/3 Human Cleric.
/// ETB: return target IS card from your gy to hand.
pub fn lorehold_recallsmith_b160() -> CardDefinition {
    use crate::effect::shortcut::etb;
    use crate::card::Zone;
    CardDefinition {
        name: "Lorehold Recallsmith (b160)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            }),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
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
    }
}

/// Lorehold Ghostflame (b160) — {2}{R} Sorcery.
/// 4 damage to target creature.
pub fn lorehold_ghostflame_b160() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ghostflame (b160)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            amount: Value::Const(4),
            to: target_filtered(SelectionRequirement::Creature),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyresage (b160) — {1}{R} 2/1 Spirit Wizard Haste.
/// Magecraft ping target creature for 1.
pub fn lorehold_pyresage_b160() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_creature;
    CardDefinition {
        name: "Lorehold Pyresage (b160)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_creature(1)],
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
    }
}

/// Lorehold Recoverer (b160) — {3}{W} 2/4 Spirit Cleric Lifelink.
/// 4-mana defensive lifelink body.
pub fn lorehold_recoverer_b160() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Recoverer (b160)",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

// ── Batch 161 (modern_decks) — More Lorehold ───────────────────────────────

/// Lorehold Pyrescholar (b161) — {2}{R}{W} 2/4 Spirit Wizard.
/// Magecraft: target opp loses 1 life and you gain 1 life.
pub fn lorehold_pyrescholar_b161() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrescholar (b161)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
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
    }
}

/// Lorehold Cavalcade (b161) — {2}{R}{W} Sorcery.
/// Create two 2/2 R/W Spirit creature tokens with haste.
pub fn lorehold_cavalcade_b161() -> CardDefinition {
    let lorehold_spirit_token = || TokenDefinition {
        name: "Spirit".to_string(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        activated_abilities: vec![] as Vec<ActivatedAbility>,
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Lorehold Cavalcade (b161)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Wallflame (b161) — {1}{R} Sorcery.
/// 3 damage to creature/PW + Surveil 1.
pub fn lorehold_wallflame_b161() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Wallflame (b161)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                amount: Value::Const(3),
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker),
                ),
            },
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Reckoner (b161) — {3}{R}{W} 4/4 Spirit Soldier Vigilance.
/// 5-mana vigilance finisher.
pub fn lorehold_reckoner_b161() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Reckoner (b161)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Spectralfist (b161) — {1}{W} 2/2 Spirit Soldier First Strike.
/// 2-mana first-strike attacker.
pub fn lorehold_spectralfist_b161() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectralfist (b161)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Pyreknight (b161) — {2}{R} 3/2 Spirit Knight Haste.
/// 3-mana hasty Spirit aggressor.
pub fn lorehold_pyreknight_b161() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyreknight (b161)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Tutorpriest (b161) — {3}{R}{W} 3/3 Spirit Cleric.
/// ETB: 2 damage to opp + gain 2 life.
pub fn lorehold_tutorpriest_b161() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Tutorpriest (b161)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(2),
        })],
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
    }
}

/// Lorehold Sparkspirit (b161) — {R}{W} 2/2 Spirit Soldier.
/// Magecraft: this creature gets +1/+0 EOT.
pub fn lorehold_sparkspirit_b161() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkspirit (b161)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Ghostbinder (b161) — {2}{W} 1/4 Spirit Cleric.
/// ETB: gain 3 life.
pub fn lorehold_ghostbinder_b161() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ghostbinder (b161)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(3)],
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
    }
}

/// Lorehold Crackleflame (b161) — {R} Instant.
/// 2 damage to target creature.
pub fn lorehold_crackleflame_b161() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Crackleflame (b161)",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            amount: Value::Const(2),
            to: target_filtered(SelectionRequirement::Creature),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 162 (modern_decks) — More Lorehold ───────────────────────────────

/// Lorehold Bonelord (b162) — {3}{R}{W} 4/4 Spirit Knight Vigilance Lifelink.
/// 5-mana finisher.
pub fn lorehold_bonelord_b162() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Bonelord (b162)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Spectralrider (b162) — {2}{R} 3/2 Spirit Knight Haste.
/// 3-mana hasty Spirit.
pub fn lorehold_spectralrider_b162() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectralrider (b162)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Brave (b162) — {1}{W} 2/2 Spirit Soldier First Strike Lifelink.
/// 2-mana defensive cleric.
pub fn lorehold_brave_b162() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Brave (b162)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Battleweave (b162) — {2}{R}{W} Sorcery.
/// 4 damage to any target + you gain 4 life.
pub fn lorehold_battleweave_b162() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battleweave (b162)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                amount: Value::Const(4),
                to: Selector::Target(0),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spectralweaver (b162) — {1}{R}{W} 2/2 Spirit Wizard.
/// ETB: 1 damage + gain 1 life.
pub fn lorehold_spectralweaver_b162() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectralweaver (b162)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage {
                amount: Value::Const(1),
                to: Selector::Target(0),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        ]))],
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
    }
}

// ── Batch 163 (modern_decks) — Lorehold Spirit cycle ──────────────────────

/// Lorehold Coursemate (b163) — {R}{W} 2/2 Spirit Cleric.
/// 2-mana hybrid-color body for an aggressive Lorehold start.
pub fn lorehold_coursemate_b163() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Coursemate (b163)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyrebound (b163) — {1}{R}{W} 3/2 Spirit Wizard Haste.
/// 3-mana hasty wizard.
pub fn lorehold_pyrebound_b163() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrebound (b163)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Spirit-Guard (b163) — {2}{R}{W} 2/4 Spirit Soldier
/// Vigilance + First Strike. Defensive lock-down.
pub fn lorehold_spirit_guard_b163() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Guard (b163)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Phantasm (b163) — {1}{W} 1/1 Spirit Flying.
/// Cheap evasive Spirit.
pub fn lorehold_phantasm_b163() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Phantasm (b163)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Sparkling Spirit (b163) — {3}{R} 3/3 Spirit Flying.
/// 4-mana evasive Spirit threat.
pub fn lorehold_sparkling_spirit_b163() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkling Spirit (b163)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Sparkscholar (b163) — {2}{R} 2/3 Spirit Wizard.
/// Magecraft ping any 1.
pub fn lorehold_sparkscholar_b163() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Lorehold Sparkscholar (b163)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Lightcage (b163) — {2}{W} Enchantment.
/// "Other Spirit creatures you control get +1/+0."
pub fn lorehold_lightcage_b163() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Lightcage (b163)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Spirit creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 164 (modern_decks) — More Lorehold ──────────────────────────────

/// Lorehold Battlemonk (b164) — {2}{R}{W} 3/3 Spirit Cleric.
/// ETB: 2 damage to target creature an opp controls + gain 2 life.
pub fn lorehold_battlemonk_b164() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battlemonk (b164)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage {
                amount: Value::Const(2),
                to: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]))],
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
    }
}

/// Lorehold Spiritforge (b164) — {3}{W} Sorcery.
/// Create two 1/1 R/W Spirit creature tokens.
pub fn lorehold_spiritforge_b164() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritforge (b164)",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: mint_lorehold_spirits(2),
        ..Default::default()
    }
}

// ── Lorehold Command ───────────────────────────────────────────────────────

/// Lorehold Command — {3}{R}{W} Sorcery. Choose two:
/// • Create a 3/2 red and white Spirit creature token.
/// • Creatures you control get +1/+0 and gain indestructible and haste EOT.
/// • Exile target nonland permanent with MV ≤ 3.
/// • Return target creature card with MV ≤ 2 from your graveyard to bf.
///
/// 🟡 Collapsed to single-mode ChooseMode of 4 modes (printed: choose two).
pub fn lorehold_command() -> CardDefinition {
    let _spirit_32 = TokenDefinition {
        name: "Spirit".into(),
        power: 3,
        toughness: 2,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    // Real Oracle: "Choose two — / • Lorehold Command deals 4 damage to
    // target player or planeswalker. / • Target player creates two 2/2
    // white and red Spirit creature tokens with flying. / …"
    //
    // Approximation: AutoDecider picks the printed default ("4 damage +
    // two flying Spirits"). Modal-choose-two is collapsed to always
    // applying both modes (Seq), which matches the gameplay outcome
    // when the controller selects those two modes.
    CardDefinition {
        name: "Lorehold Command",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(4),
            },
            mint_lorehold_spirits(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyremender (b164) — {1}{R}{W} 2/2 Human Cleric.
/// Magecraft: gain 1 life and 1 damage to target creature or player.
pub fn lorehold_pyremender_b164() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyremender (b164)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            Effect::DealDamage {
                amount: Value::Const(1),
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
            },
        ]))],
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
    }
}

/// Lorehold Ghostflame (b164) — {2}{R} Sorcery.
/// 3 damage to target creature; if it dies this turn, return it to gy as
/// a Spirit (approximated: just deal 3 damage).
pub fn lorehold_ghostflame_b164() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ghostflame (b164)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            amount: Value::Const(3),
            to: target_filtered(SelectionRequirement::Creature),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Skybinder (b164) — {3}{W}{R} 3/4 Spirit Soldier Flying Vigilance.
/// Top-end evasive defender.
pub fn lorehold_skybinder_b164() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skybinder (b164)",
        cost: cost(&[generic(3), w(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Ironforge (b164) — {R}{W} 2/2 Spirit Warrior First Strike.
/// Aggressive 2-mana hybrid Spirit.
pub fn lorehold_ironforge_b164() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ironforge (b164)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Spectralward (b164) — {R}{W} Instant.
/// Target creature gets +1/+1 EOT and gains lifelink until end of turn.
pub fn lorehold_spectralward_b164() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectralward (b164)",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── Academic Dispute ───────────────────────────────────────────────────────

/// Academic Dispute — {R} Instant. "Target creature gets +2/+0 and gains
/// reach until end of turn. It must be blocked this turn if able."
///
/// 🟡 "Must be blocked" rider is omitted (no forced-block primitive).
pub fn academic_dispute() -> CardDefinition {
    CardDefinition {
        name: "Academic Dispute",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Reach,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spiritcaller (b164) — {3}{R}{W} 2/3 Spirit Wizard.
/// Magecraft mint a 2/2 R/W Spirit token (lorehold spirit).
pub fn lorehold_spiritcaller_b164() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritcaller (b164)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![magecraft(mint_lorehold_spirits(1))],
        ..Default::default()
    }
}

// ── Blade Historian ────────────────────────────────────────────────────────

/// Blade Historian — {R}{R}{W}{W}, 2/3 Human Cleric.
/// "Attacking creatures you control have double strike."
///
/// 🟡 The continuous static granting double strike to attackers needs a
/// layer-based keyword-grant primitive. We ship the body only.
#[allow(dead_code)]
pub fn blade_historian() -> CardDefinition {
    CardDefinition {
        name: "Blade Historian",
        cost: cost(&[r(), r(), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(mint_lorehold_spirits(1))],
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
    }
}

// ── Batch 165 (modern_decks) — More Lorehold ──────────────────────────────

/// Lorehold Flamebinder (b165) — {1}{R} 2/1 Spirit Wizard Haste.
pub fn lorehold_flamebinder_b165() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Flamebinder (b165)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Sunweave (b165) — {3}{W} Sorcery.
/// Gain 5 life + Scry 1.
pub fn lorehold_sunweave_b165() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sunweave (b165)",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(5),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

// ── Reconstruct History ────────────────────────────────────────────────────

/// Lorehold Pyreguard (b165) — {2}{R}{W} 3/3 Spirit Soldier Vigilance.
/// ETB: 2 damage to target player.
pub fn lorehold_pyreguard_b165() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyreguard (b165)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::DealDamage {
            amount: Value::Const(2),
            to: Selector::Target(0),
        })],
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
    }
}

/// Lorehold Braveheart (b165) — {1}{W} 2/2 Spirit Cleric Lifelink.
pub fn lorehold_braveheart_b165() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Braveheart (b165)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Fireshield (b165) — {R}{W} Instant.
/// Target creature gets +2/+2 and gains first strike EOT.
pub fn lorehold_fireshield_b165() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Fireshield (b165)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── Rip Apart ──────────────────────────────────────────────────────────────

/// Rip Apart — {R}{W} Sorcery. "Choose one — • Rip Apart deals 3 damage
/// to target creature or planeswalker. • Destroy target artifact or
/// enchantment."
pub fn rip_apart() -> CardDefinition {
    CardDefinition {
        name: "Rip Apart",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Enchantment),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Bonepreacher (b165) — {3}{R}{W} 4/3 Spirit Cleric Flying.
/// ETB: gain 3 life.
pub fn lorehold_bonepreacher_b165() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Lorehold Bonepreacher (b165)",
        cost: cost(&[generic(3), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
        }],
        ..Default::default()
    }
}

// ── Batch 166 (modern_decks) — Lorehold cycle ─────────────────────────────
//
// Ten new Lorehold (R/W) cards: a mix of Spirit minters, magecraft ping,
// attack triggers, and graveyard recursion. Each composes against
// existing shortcuts.

/// Lorehold Sparkmage (b166) — {R}{W} 2/2 Human Wizard.
/// Magecraft: deals 1 damage to any target.
pub fn lorehold_sparkmage_b166() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkmage (b166)",
        cost: cost(&[r(), w()]),
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
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Spiritskirmisher (b166) — {1}{R} 2/1 Spirit Soldier Haste.
/// Aggressive haste body.
pub fn lorehold_spiritskirmisher_b166() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritskirmisher (b166)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Pyresmith (b166) — {2}{R} 3/2 Human Warrior.
/// On-attack: deals 1 damage to target opp creature.
pub fn lorehold_pyresmith_b166() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Lorehold Pyresmith (b166)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Recall (b166) — {2}{W} Sorcery.
/// Return target creature card from your gy → hand + gain 2 life.
pub fn lorehold_recall_b166() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Recall (b166)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    },
                    Value::Const(1),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Pyreweaver (b166) — {2}{R}{W} 3/3 Spirit Warrior.
/// ETB mints 1 Lorehold Spirit token.
pub fn lorehold_pyreweaver_b166() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyreweaver (b166)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(mint_lorehold_spirits(1))],
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
    }
}

/// Lorehold Vandal (b166) — {3}{R} 3/3 Human Warrior.
/// ETB: destroy target artifact.
pub fn lorehold_vandal_b166() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Vandal (b166)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(SelectionRequirement::Artifact),
        })],
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
    }
}

/// Lorehold Spectrescholar (b166) — {2}{W} 2/3 Spirit Cleric Vigilance.
/// Magecraft gain 1 life.
pub fn lorehold_spectrescholar_b166() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectrescholar (b166)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
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
    }
}

/// Lorehold Charge (b166) — {1}{R}{W} Sorcery.
/// Mints 2 Lorehold Spirit tokens.
pub fn lorehold_charge_b166() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Charge (b166)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: mint_lorehold_spirits(2),
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Boltmage (b166) — {R} Instant.
/// 2 damage to any target.
pub fn lorehold_boltmage_b166() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Boltmage (b166)",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Battlespirit (b166) — {3}{R}{W} 4/4 Spirit Warrior
/// Flying + Haste. 5-mana evasive aggro finisher.
pub fn lorehold_battlespirit_b166() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battlespirit (b166)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

// ── Batch 167 (modern_decks) — Lorehold follow-up ─────────────────────────

/// Lorehold Banisher (b167) — {3}{W} Sorcery.
/// Exile target creature with mana value ≤ 3.
pub fn lorehold_banisher_b167() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Banisher (b167)",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMost(3)),
            ),
            to: ZoneDest::Exile,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Champion (b167) — {3}{R}{W} 3/3 Spirit Warrior First Strike +
/// Vigilance. Aggressive defensive top-end.
pub fn lorehold_champion_b167() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Champion (b167)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Strikewing (b167) — {2}{R} 2/3 Spirit Wizard Flying.
/// Aggressive evasive 3-mana flyer.
pub fn lorehold_strikewing_b167() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Strikewing (b167)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Inscription (b167) — {R}{W} Instant.
/// 2 damage to target creature + you gain 2 life.
pub fn lorehold_inscription_b167() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Inscription (b167)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(2),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spiritcaller II (b167) — {2}{W} 2/3 Spirit Cleric.
/// ETB mints a Lorehold Spirit token.
pub fn lorehold_spiritcaller_ii_b167() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritcaller II (b167)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(mint_lorehold_spirits(1))],
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
    }
}

// ── Returned Pastcaller ───────────────────────────────────────────────────

// ── Batch 169 (modern_decks) — Lorehold expansion (8 cards) ───────────────

/// Lorehold Sparkblade (b169) — {2}{R}{W} 3/3 Human Warrior.
/// Magecraft: this creature gets +1/+1 EOT.
pub fn lorehold_sparkblade_b169() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkblade (b169)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
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
        ..Default::default()
    }
}

/// Lorehold Spiritforge (b169) — {3}{R}{W} 3/3 Human Cleric Vigilance.
/// ETB: create a 2/2 R/W Spirit token with reach.
pub fn lorehold_spiritforge_b169() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritforge (b169)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(mint_lorehold_spirits(1))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Reciter (b169) — {1}{R} 2/1 Human Cleric.
/// Magecraft: target creature an opponent controls gets -1/-0 EOT.
pub fn lorehold_reciter_b169() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Reciter (b169)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Const(-1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Reverence (b169) — {1}{R}{W} Sorcery.
/// Create two 2/2 R/W Spirit tokens with reach.
pub fn lorehold_reverence_b169() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Reverence (b169)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: mint_lorehold_spirits(2),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Lectern (b169) — {3} Artifact.
/// Whenever you cast an instant or sorcery spell, you may pay {1} to scry 1.
/// Approximation: magecraft scry 1 unconditionally (no engine "may pay {1}").
pub fn lorehold_lectern_b169() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Lectern (b169)",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_scry(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Quartermaster (b169) — {2}{R} 2/3 Dwarf Warrior.
/// Whenever this creature attacks, deal 1 damage to any target.
pub fn lorehold_quartermaster_b169() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Quartermaster (b169)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_ping_any(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Flameglyph (b169) — {1}{R} Instant.
/// Deal 3 damage to target creature.
pub fn lorehold_flameglyph_b169() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Flameglyph (b169)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(3),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

// ── Batch 170 (modern_decks) — Shield-counter cards (CR 122.1c wire) ─────

/// Lorehold Shieldbearer (b170) — {2}{R}{W} 2/2 Spirit Soldier.
/// ETB: put a shield counter on this creature. (CR 122.1c: one or more
/// shield counters create a destroy-replacement + damage-prevention.)
pub fn lorehold_shieldbearer_b170() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Shieldbearer (b170)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Shield,
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

// ── Batch 171 (modern_decks) — Lorehold expansion ─────────────────────────

/// Lorehold Skirmisher (b171) — {1}{R} 2/2 Human Soldier Haste.
/// Vanilla aggressive 2-drop.
pub fn lorehold_skirmisher_b171() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skirmisher (b171)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

// ── Batch 172 (modern_decks) — Lorehold expansion ─────────────────────────

// ── Batch 173 (modern_decks) — Shield/finality magecraft variants ─────────

/// Lorehold Wardseeker (b173) — {2}{R}{W} 2/2 Spirit Warrior.
/// Magecraft: put a shield counter on this creature.
pub fn lorehold_wardseeker_b173() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_shield_self;
    CardDefinition {
        name: "Lorehold Wardseeker (b173)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_add_shield_self()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Embersmith (b172) — {2}{R} 3/2 Dwarf Soldier.
/// Magecraft: deal 1 damage to target opp.
pub fn lorehold_embersmith_b172() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Embersmith (b172)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Pyresage (b171) — {2}{R}{W} 2/4 Human Cleric.
/// Magecraft: deal 1 damage to any target.
pub fn lorehold_pyresage_b171() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyresage (b171)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Aegisblade (b170) — {2}{W} Sorcery.
/// Put a shield counter on target creature.
pub fn lorehold_aegisblade_b170() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Aegisblade (b170)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::Shield,
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
        ..Default::default()
    }
}

/// Lorehold Aurochs (b169) — {3}{R}{W} 4/4 Beast Spirit Trample.
/// Vanilla R/W finisher.
pub fn lorehold_aurochs_b169() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Aurochs (b169)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast, CreatureType::Spirit],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
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
        ..Default::default()
    }
}

// ── Batch 174 (modern_decks) — additional Lorehold cards ──────────────────

/// Lorehold Pyrespirit (b174) — {1}{R} 2/1 Spirit. Haste.
/// Magecraft: deal 1 damage to any target.
pub fn lorehold_pyrespirit_b174() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrespirit (b174)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Banneret (b174) — {2}{R}{W} 2/4 Spirit Soldier.
/// ETB: gain 2 life. Vigilance.
pub fn lorehold_banneret_b174() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Banneret (b174)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Sparkborn (b174) — {1}{R} 1/2 Spirit.
/// Whenever this creature attacks, deal 1 damage to any target.
pub fn lorehold_sparkborn_b174() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkborn (b174)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_ping_any(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Ghostflame (b174) — {1}{R}{W} Instant.
/// Deal 2 damage to target creature; gain 2 life.
pub fn lorehold_ghostflame_b174() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Lorehold Ghostflame (b174)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(2),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
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
        ..Default::default()
    }
}

/// Lorehold Spectralcaller (b174) — {3}{R}{W} 3/3 Spirit Wizard.
/// ETB: create a 1/1 R/W Spirit (uses lorehold_spirit_token shape — actually a 2/2 Spirit Flying via the existing helper).
pub fn lorehold_spectralcaller_b174() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectralcaller (b174)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(mint_lorehold_spirits(1))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

// ── Batch 175 (modern_decks) — additional Lorehold cards ──────────────────

/// Lorehold Skirmishmage (b175) — {2}{R}{W} 3/2 Human Cleric.
/// On attack: loot (draw a card, then discard a card).
pub fn lorehold_skirmishmage_b175() -> CardDefinition {
    use crate::effect::shortcut::on_attack_loot;
    CardDefinition {
        name: "Lorehold Skirmishmage (b175)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Anthemwarden (b175) — {3}{R}{W} 2/4 Spirit Soldier.
/// Other Spirits you control get +1/+1.
pub fn lorehold_anthemwarden_b175() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Lorehold Anthemwarden (b175)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Spirits you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

// ── Batch 191 (modern_decks) — multi-action cards + Spirit tribal ─────────

/// Lorehold Echobringer (b191) — {3}{R}{W} Sorcery.
/// Mints 2 Lorehold Spirits + deals 2 to any target.
pub fn lorehold_echobringer_b191() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Echobringer (b191)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            mint_lorehold_spirits(2),
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(2),
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
        ..Default::default()
    }
}

/// Lorehold Sparrowscholar (b191) — {1}{W} 1/2 Spirit Cleric.
/// Magecraft Scry 1 + draw 0 (placeholder for the simple body).
pub fn lorehold_sparrowscholar_b191() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparrowscholar (b191)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_scry(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Embershield (b191) — {2}{W} 2/3 Spirit Soldier Vigilance.
/// First Strike.
pub fn lorehold_embershield_b191() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Embershield (b191)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

// ── Batch 190 (modern_decks) — keyword counter granters ──────────────────

/// Lorehold Doubleblast (b190) — {2}{R} Sorcery.
/// Target creature gets a first strike counter and a haste counter.
pub fn lorehold_doubleblast_b190() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Doubleblast (b190)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddKeywordCounter {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::FirstStrike,
                amount: Value::Const(1),
            },
            Effect::AddKeywordCounter {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Haste,
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
        ..Default::default()
    }
}

/// Lorehold Bondseal (b190) — {1}{W} Sorcery.
/// Target creature gets a vigilance counter and a +1/+1 counter.
pub fn lorehold_bondseal_b190() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Bondseal (b190)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddKeywordCounter {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Vigilance,
                amount: Value::Const(1),
            },
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
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
        ..Default::default()
    }
}

/// Lorehold Phoenixmage (b190) — {2}{R} 3/2 Phoenix.
/// ETB self-haste counter (flies through self-keyword wire) — flavor: a
/// Phoenix that hastes itself.
pub fn lorehold_phoenixmage_b190() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Phoenixmage (b190)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phoenix],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::AddKeywordCounter {
            what: Selector::This,
            keyword: Keyword::Haste,
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

// ── Batch 189 (modern_decks) — additional Lorehold cards ──────────────────

/// Lorehold Voltmage (b189) — {2}{R} 2/2 Spirit Wizard.
/// ETB ping 2 to any target.
pub fn lorehold_voltmage_b189() -> CardDefinition {
    use crate::effect::shortcut::etb_ping_any;
    CardDefinition {
        name: "Lorehold Voltmage (b189)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_ping_any(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Fireseal (b189) — {2}{R}{W} Sorcery.
/// Mints 2 Lorehold Spirits + grant Haste EOT.
pub fn lorehold_fireseal_b189() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Fireseal (b189)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            mint_lorehold_spirits(2),
            Effect::GrantKeyword {
                what: Selector::LastCreatedToken,
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
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
        ..Default::default()
    }
}

/// Lorehold Crusader (b189) — {1}{R}{W} 2/3 Spirit Knight Vigilance.
/// Magecraft self-pump +1/+0 EOT.
pub fn lorehold_crusader_b189() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Crusader (b189)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

// ── Batch 188 (modern_decks) — additional Lorehold cards ──────────────────

/// Lorehold Spiritsong (b188) — {1}{R}{W} 2/2 Spirit Cleric Lifelink.
/// Magecraft self-pump +1/+1 EOT.
pub fn lorehold_spiritsong_b188() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritsong (b188)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Sparkbarrier (b188) — {2}{R} Sorcery.
/// 3 damage to any target + create a 1/1 R/W Spirit (Lorehold spirit token).
pub fn lorehold_sparkbarrier_b188() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkbarrier (b188)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                amount: Value::Const(3),
                to: Selector::Target(0),
            },
            mint_lorehold_spirits(1),
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Vanguard II (b188) — {2}{R}{W} 3/3 Spirit Soldier.
/// Vigilance + Reach + magecraft self-pump +1/+0 EOT.
pub fn lorehold_vanguard_ii_b188() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Vanguard II (b188)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

// ── Batch 187 (modern_decks) — Lorehold expansion ─────────────────────────

/// Lorehold Firstrikedoctrine (b187) — {1}{R}{W} Sorcery.
/// Put a first strike counter on target creature you control.
pub fn lorehold_firstrikedoctrine_b187() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Firstrikedoctrine (b187)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddKeywordCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            keyword: Keyword::FirstStrike,
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
        ..Default::default()
    }
}

/// Lorehold Battleseer (b187) — {2}{R}{W} 3/3 Spirit Cleric First Strike.
/// Magecraft: target friendly creature gets +1/+1 EOT.
pub fn lorehold_battleseer_b187() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battleseer (b187)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
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
        ..Default::default()
    }
}

/// Lorehold Memorymage (b187) — {3}{R}{W} 3/4 Spirit Cleric.
/// ETB: return target IS card from gy → hand.
pub fn lorehold_memorymage_b187() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Memorymage (b187)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            }),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Spiritcaller (b187) — {1}{R}{W} 2/2 Spirit Cleric.
/// Whenever a creature you control dies, mint a Lorehold Spirit (1/1 R/W).
pub fn lorehold_spiritcaller_b187() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritcaller (b187)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_other_dies_mint_token(lorehold_spirit_token(), 1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Pyrescribe (b187) — {1}{R} Instant.
/// Deal 2 damage to any target + scry 1.
pub fn lorehold_pyrescribe_b187() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrescribe (b187)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                amount: Value::Const(2),
                to: Selector::Target(0),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Ghostpaladin (b187) — {2}{W} 2/3 Spirit Knight Vigilance.
/// ETB tap target opp creature.
pub fn lorehold_ghostpaladin_b187() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ghostpaladin (b187)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Tap {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Reach Doctrine (b187) — {2}{R} Sorcery.
/// Put a reach counter on target creature.
pub fn lorehold_reach_doctrine_b187() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Reach Doctrine (b187)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddKeywordCounter {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::Reach,
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
        ..Default::default()
    }
}

// ── Batch 184 (modern_decks) — more keyword counter cards ─────────────────

/// Lorehold Battlerune (b184) — {2}{R}{W} Sorcery.
/// Put a haste counter on target creature.
pub fn lorehold_battlerune_b184() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Lorehold Battlerune (b184)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddKeywordCounter {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::Haste,
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
        ..Default::default()
    }
}

/// Lorehold Wardseal (b184) — {1}{W} Sorcery.
/// Put a vigilance counter on target creature.
pub fn lorehold_wardseal_b184() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Lorehold Wardseal (b184)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddKeywordCounter {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::Vigilance,
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
        ..Default::default()
    }
}

// ── Batch 182 (modern_decks) — closer to a balanced Lorehold cube ─────────

/// Lorehold Cinderwell (b182) — {2}{R} 3/2 Spirit Warrior.
/// On unblocked attack: deal 1 damage to defending player.
pub fn lorehold_cinderwell_b182() -> CardDefinition {
    use crate::effect::shortcut::on_unblocked;
    CardDefinition {
        name: "Lorehold Cinderwell (b182)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_unblocked(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

// ── Batch 180 (modern_decks) — Lorehold Spirit-tribal expansion ──────────

/// Lorehold Spiritlord (b180) — {3}{R}{W} 3/3 Spirit Soldier.
/// ETB: create two 2/2 R/W Spirit tokens with flying.
pub fn lorehold_spiritlord_b180() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritlord (b180)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(mint_lorehold_spirits(2))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Spectralguard (b180) — {2}{W} 2/4 Spirit Cleric.
/// Vigilance + on-attack gain 1 life.
pub fn lorehold_spectralguard_b180() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectralguard (b180)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_gain_life(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

// ── Batch 178 (modern_decks) — more Lorehold variants ─────────────────────

/// Lorehold Sparkscholar (b178) — {1}{R} 1/3 Spirit Wizard.
/// {3}{R}: deal 2 damage to any target. (Cleanup activation.)
pub fn lorehold_sparkscholar_b178() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Lorehold Sparkscholar (b178)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(3), r()]),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

// ── Batch 177 (modern_decks) — more Lorehold variants ─────────────────────

/// Lorehold Ghostsmith (b177) — {2}{W} 1/4 Spirit Cleric.
/// Whenever you cast an instant or sorcery, target creature gets +1/+1 EOT.
pub fn lorehold_ghostsmith_b177() -> CardDefinition {
    use crate::effect::shortcut::{magecraft_target_pump, target_filtered};
    CardDefinition {
        name: "Lorehold Ghostsmith (b177)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou)),
            1,
            1,
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Cultivator (b177) — {3}{R} 4/3 Human Berserker.
/// Magecraft: deal 1 damage to each opp.
pub fn lorehold_cultivator_b177() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Cultivator (b177)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Berserker],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Charm-Echo (b175) — {1}{R} Instant.
/// Deal 3 damage to target creature.
pub fn lorehold_charm_echo_b175() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Lorehold Charm-Echo (b175)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(3),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Lorehold Vanguard (b174) — {3}{R}{W} 4/4 Spirit Knight.
/// Trample. Vanilla mid-curve attacker.
pub fn lorehold_vanguard_b174() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Vanguard (b174)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
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
        ..Default::default()
    }
}

/// Returned Pastcaller — {4}{W}, 4/4 Spirit Cleric. Flying.
/// ETB: "Return target instant or sorcery card from your graveyard to
/// your hand." Same shape as Pillardrop Rescuer.
pub fn returned_pastcaller() -> CardDefinition {
    CardDefinition {
        name: "Returned Pastcaller",
        cost: cost(&[generic(4), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
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
    }
}


// ── Batch 193 (modern_decks) — Lorehold R/W deep cuts ────────────────────

/// Lorehold Stoneward (b193) — {1}{W} 1/3 Spirit Soldier Vigilance.
/// Cheap defensive Spirit body.
pub fn lorehold_stoneward_b193() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Stoneward (b193)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Ravenrider (b193) — {2}{R} 3/2 Human Warrior Haste.
/// Cheap aggressive Lorehold body.
pub fn lorehold_ravenrider_b193() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ravenrider (b193)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Boltstudent (b193) — {R} Instant.
/// Deal 2 damage to target creature.
pub fn lorehold_boltstudent_b193() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Boltstudent (b193)",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spiritsummoner (b193) — {3}{R}{W} 3/3 Cleric.
/// ETB: create two 2/2 Spirit tokens.
pub fn lorehold_spiritsummoner_b193() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritsummoner (b193)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(mint_lorehold_spirits(2))],
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
    }
}

/// Lorehold Pyrescholar (b193) — {1}{R}{W} 2/2 Human Cleric.
/// Magecraft: ping target creature or player for 1.
pub fn lorehold_pyrescholar_b193() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyrescholar (b193)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Sparkscholar (b193) — {1}{R} 2/1 Human Wizard.
/// Magecraft: source gets +1/+0 EOT.
pub fn lorehold_sparkscholar_b193() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkscholar (b193)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Soulsign (b193) — {1}{W} Instant.
/// Target creature gains lifelink and vigilance until end of turn.
pub fn lorehold_soulsign_b193() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Soulsign (b193)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 194 (modern_decks) — Lorehold R/W compact additions ─────────────

/// Lorehold Boltscribe (b194) — {2}{R} 3/2 Human Wizard.
/// On attack: deal 1 damage to any target.
pub fn lorehold_boltscribe_b194() -> CardDefinition {
    use crate::effect::shortcut::on_attack_ping_any;
    CardDefinition {
        name: "Lorehold Boltscribe (b194)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_ping_any(1)],
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
    }
}

/// Lorehold Vanguard II (b194) — {1}{W} 2/2 Spirit Soldier First Strike.
/// Cheap Spirit aggro body.
pub fn lorehold_vanguard_ii_b194() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Vanguard II (b194)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Bolt (b194) — {R} Instant. Deal 3 damage to any target.
pub fn lorehold_bolt_b194() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Bolt (b194)",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(3),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Mausolescholar (b194) — {1}{R}{W} 2/2 Cleric.
/// On attack: create a Lorehold Spirit (2/2).
pub fn lorehold_mausolescholar_b194() -> CardDefinition {
    use crate::effect::shortcut::on_attack_mint_lorehold_spirit;
    CardDefinition {
        name: "Lorehold Mausolescholar (b194)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_mint_lorehold_spirit()],
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
    }
}

// ── Batch 195 (modern_decks) — Lorehold more deep cuts ────────────────────

/// Lorehold Frostbreaker (b195) — {2}{R} 3/3 Spirit.
/// On attack: deal 1 damage to each opp.
pub fn lorehold_frostbreaker_b195() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Frostbreaker (b195)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spiritcaller (b195) — {3}{R}{W} 4/4 Cleric.
/// ETB: create a 2/2 Spirit token. Magecraft: gain 1 life.
pub fn lorehold_spiritcaller_b195() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritcaller (b195)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb(mint_lorehold_spirits(1)),
            magecraft_gain_life(1),
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
    }
}

/// Lorehold Memoryguard (b195) — {1}{W} 2/2 Spirit Cleric Lifelink.
/// Cheap Spirit lifegain body.
pub fn lorehold_memoryguard_b195() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Memoryguard (b195)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Pyrescribe (b195) — {3}{R} 3/3 Human Wizard.
/// ETB: deal 2 damage to any target.
pub fn lorehold_pyrescribe_b195() -> CardDefinition {
    use crate::effect::shortcut::etb_ping_any;
    CardDefinition {
        name: "Lorehold Pyrescribe (b195)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_ping_any(2)],
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
    }
}

// ── Batch 196 (modern_decks) — Lorehold more variety ─────────────────────

/// Lorehold Lavalord (b196) — {3}{R} 4/3 Elemental.
/// On attack: deal 2 damage to any target.
pub fn lorehold_lavalord_b196() -> CardDefinition {
    use crate::effect::shortcut::on_attack_ping_any;
    CardDefinition {
        name: "Lorehold Lavalord (b196)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_ping_any(2)],
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
    }
}

/// Lorehold Sparkward (b196) — {1}{W} Instant.
/// Target creature gets +1/+3 EOT.
pub fn lorehold_sparkward_b196() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkward (b196)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(1),
            toughness: Value::Const(3),
            duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Stormrider (b196) — {3}{R}{W} 3/3 Spirit Soldier.
/// First strike, vigilance. ETB: create a 2/2 Spirit token.
pub fn lorehold_stormrider_b196() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Stormrider (b196)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(mint_lorehold_spirits(1))],
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
    }
}

/// Lorehold Bookburn (b196) — {2}{R} Sorcery.
/// Deal 4 damage divided as you choose among any number of target creatures.
/// (Approximated as 4 damage to a single creature.)
pub fn lorehold_bookburn_b196() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Bookburn (b196)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(4),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 197 (modern_decks) — Lorehold polish ───────────────────────────

/// Lorehold Sparkmage (b197) — {1}{R} 2/1 Human Wizard.
/// Magecraft: deal 1 damage to any target.
pub fn lorehold_sparkmage_b197() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkmage (b197)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Brawnsage (b197) — {2}{R}{W} 4/4 Spirit Warrior.
/// Vanilla 4/4 in school colors.
pub fn lorehold_brawnsage_b197() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Brawnsage (b197)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
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
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 198 (modern_decks) — Lorehold (R/W) extension ──────────────────

/// Lorehold Apprentice II (b198) — {R}{W} 2/2 Human Wizard.
/// Magecraft: target creature gets +1/+1 EOT (Eager-First-Year template).
pub fn lorehold_apprentice_ii_b198() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Lorehold Apprentice II (b198)",
        cost: cost(&[r(), w()]),
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
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(SelectionRequirement::Creature),
            1,
            1,
        )],
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
    }
}

/// Lorehold Watchtower (b198) — {2}{W} 1/4 Soldier Vigilance.
/// Defensive vigilance body.
pub fn lorehold_watchtower_b198() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Watchtower (b198)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Pyromancer (b198) — {2}{R} 2/2 Human Shaman.
/// Magecraft: deal 1 damage to each opponent.
pub fn lorehold_pyromancer_b198() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_each_opp;
    CardDefinition {
        name: "Lorehold Pyromancer (b198)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_each_opp(1)],
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
    }
}

/// Lorehold Sparkbinder (b198) — {1}{R} Instant.
/// Deal 2 damage to any target.
pub fn lorehold_sparkbinder_b198() -> CardDefinition {
    use crate::effect::shortcut::target_any;
    CardDefinition {
        name: "Lorehold Sparkbinder (b198)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_any(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Battlecaller (b198) — {2}{R}{W} 3/3 Spirit Warrior.
/// On attack: target creature gets +1/+1 EOT.
pub fn lorehold_battlecaller_b198() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Lorehold Battlecaller (b198)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spiritbinder (b198) — {3}{W} 2/3 Cleric.
/// ETB mints a Lorehold Spirit (1/2 White Spirit) token.
pub fn lorehold_spiritbinder_b198() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritbinder (b198)",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(mint_lorehold_spirits(1))],
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
    }
}

/// Lorehold Burner (b198) — {3}{R} Sorcery.
/// Deal 3 damage to each opponent.
pub fn lorehold_burner_b198() -> CardDefinition {
    use crate::effect::shortcut::each_opponent;
    CardDefinition {
        name: "Lorehold Burner (b198)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: each_opponent(),
            amount: Value::Const(3),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Champion (b198) — {3}{R}{W} 4/4 Spirit Soldier First Strike.
/// First-strike top-end Spirit.
pub fn lorehold_champion_b198() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Champion (b198)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

// ── Batch 199 (modern_decks) — Lorehold rounding-out ─────────────────────

/// Lorehold Ember (b199) — {R} Instant.
/// Deal 1 damage to any target. Cantrip-tier sparker.
pub fn lorehold_ember_b199() -> CardDefinition {
    use crate::effect::shortcut::target_any;
    CardDefinition {
        name: "Lorehold Ember (b199)",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_any(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Recurrence (b199) — {2}{W} Sorcery.
/// Return target creature card from your graveyard to your hand.
pub fn lorehold_recurrence_b199() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Recurrence (b199)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature,
            }),
            to: ZoneDest::Hand(PlayerRef::You),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Frontier (b199) — {3}{R}{W} 3/3 Spirit Warrior Trample.
/// Vanilla trample Spirit.
pub fn lorehold_frontier_b199() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Frontier (b199)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Boltscribe (b199) — {1}{R} 2/1 Wizard.
/// Magecraft self-pump (+1/+0 EOT).
pub fn lorehold_boltscribe_b199() -> CardDefinition {
    use crate::effect::shortcut::magecraft_self_pump;
    CardDefinition {
        name: "Lorehold Boltscribe (b199)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

/// Lorehold Aegis (b199) — {3}{W} 2/4 Soldier Vigilance.
/// Mid-defender body.
pub fn lorehold_aegis_b199() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Aegis (b199)",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

// ── Batch 200 (modern_decks) — Lorehold round 200 ───────────────────────

/// Lorehold Sparkguard (b200) — {2}{R} 3/2 Spirit Soldier.
pub fn lorehold_sparkguard_b200() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkguard (b200)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Smite (b200) — {1}{W} Sorcery.
/// Destroy target tapped creature.
pub fn lorehold_smite_b200() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Smite (b200)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
            ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 201 (modern_decks) — Lorehold nuanced round ────────────────────

/// Lorehold Vanguard (b201) — {2}{R}{W} 3/3 Spirit Soldier Vigilance.
/// On attack: target creature you control gets +1/+1 EOT.
pub fn lorehold_vanguard_b201() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Lorehold Vanguard (b201)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack(Effect::PumpPT {
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Wildfire (b201) — {3}{R}{R} Sorcery.
/// Deal 3 damage to each creature.
pub fn lorehold_wildfire_b201() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Wildfire (b201)",
        cost: cost(&[generic(3), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: Selector::EachPermanent(SelectionRequirement::Creature),
            amount: Value::Const(3),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 202 (modern_decks) — Lorehold expansion ────────────────────────

/// Lorehold Reanimator (b202) — {3}{R}{W} 3/3 Spirit Soldier.
/// ETB: return target creature card with mana value 3 or less from
/// your graveyard to the battlefield.
pub fn lorehold_reanimator_b202() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Reanimator (b202)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMost(3)),
            }),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        })],
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
    }
}

/// Lorehold Pyromancer (b202) — {2}{R} 2/2 Human Wizard.
/// Magecraft: deal 2 damage to any target. Burn-engine on a body.
pub fn lorehold_pyromancer_b202() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Lorehold Pyromancer (b202)",
        cost: cost(&[generic(2), r()]),
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
        triggered_abilities: vec![magecraft_ping_any(2)],
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
    }
}

/// Lorehold Charge (b202) — {1}{R}{W} Sorcery.
/// Creatures you control get +1/+0 and gain first strike until EOT.
pub fn lorehold_charge_b202() -> CardDefinition {
    use crate::effect::shortcut::each_your_creature;
    CardDefinition {
        name: "Lorehold Charge (b202)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: each_your_creature(),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: each_your_creature(),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spirit Caller (b202) — {2}{R}{W} 2/2 Spirit Cleric.
/// On attack: mint a 2/2 R/W Spirit token. Recursive go-wide attacker.
pub fn lorehold_spirit_caller_b202() -> CardDefinition {
    use crate::effect::shortcut::on_attack_create_token;
    CardDefinition {
        name: "Lorehold Spirit Caller (b202)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_create_token(lorehold_spirit_token())],
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
    }
}

/// Lorehold Bolt II (b202) — {R} Instant.
/// Deals 2 damage to any target.
pub fn lorehold_bolt_ii_b202() -> CardDefinition {
    use crate::effect::shortcut::target_any;
    CardDefinition {
        name: "Lorehold Bolt II (b202)",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Battlescholar (b202) — {R}{W} 2/2 Spirit Soldier.
/// First Strike. Aggressive 2-drop.
pub fn lorehold_battlescholar_b202() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battlescholar (b202)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Excavate (b202) — {2}{R}{W} Sorcery.
/// Return target creature or planeswalker card from your graveyard to
/// the battlefield. Reanimation spell at 4 mana.
pub fn lorehold_excavate_b202() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Excavate (b202)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature,
            }),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Frontlord (b202) — {3}{R}{W} 4/4 Spirit Soldier.
/// Vigilance. Static "Other creatures you control get +1/+0."
pub fn lorehold_frontlord_b202() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Frontlord (b202)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Cleanse (b202) — {2}{R}{W} Sorcery.
/// Deal 2 damage to each creature.
pub fn lorehold_cleanse_b202() -> CardDefinition {
    use crate::effect::shortcut::each_creature;
    CardDefinition {
        name: "Lorehold Cleanse (b202)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage { to: each_creature(), amount: Value::Const(2) },
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Echoblade (b202) — {1}{R}{W} 2/3 Spirit Cleric.
/// Magecraft: target creature you control gets +1/+1 EOT.
pub fn lorehold_echoblade_b202() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Lorehold Echoblade (b202)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            1, 1,
        )],
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
    }
}

/// Lorehold Lavascholar (b202) — {2}{R} 3/2 Human Wizard.
/// ETB: deal 1 damage to any target. Aggressive ping body.
pub fn lorehold_lavascholar_b202() -> CardDefinition {
    use crate::effect::shortcut::etb_ping_any;
    CardDefinition {
        name: "Lorehold Lavascholar (b202)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_ping_any(1)],
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
    }
}

/// Lorehold Ghostsmith (b202) — {2}{R}{W} 3/3 Spirit Warrior.
/// Whenever this creature attacks, mint a 2/2 R/W Spirit token.
pub fn lorehold_ghostsmith_b202() -> CardDefinition {
    use crate::effect::shortcut::on_attack_create_token;
    CardDefinition {
        name: "Lorehold Ghostsmith (b202)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_create_token(lorehold_spirit_token())],
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
    }
}

// ── Batch 203 (modern_decks) — Lorehold compact round ────────────────────

/// Lorehold Apprentice (b203) — {R}{W} 2/2 Spirit. Magecraft ping 1.
pub fn lorehold_apprentice_b203() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Apprentice (b203)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Soulbinder (b203) — {3}{R}{W} 3/4 Spirit Cleric Vigilance.
/// On attack: mint a Lorehold Spirit token.
pub fn lorehold_soulbinder_b203() -> CardDefinition {
    use crate::effect::shortcut::on_attack_create_token;
    CardDefinition {
        name: "Lorehold Soulbinder (b203)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack_create_token(lorehold_spirit_token())],
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
    }
}

/// Lorehold Spirit Sage (b203) — {2}{W} 2/3 Spirit Cleric. ETB mint a
/// Lorehold spirit token.
pub fn lorehold_spirit_sage_b203() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit Sage (b203)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lorehold_spirit_token(),
        })],
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
    }
}

/// Lorehold Strike (b203) — {1}{R} Instant. Deal 3 damage to any target.
pub fn lorehold_strike_b203() -> CardDefinition {
    use crate::effect::shortcut::target_any;
    CardDefinition {
        name: "Lorehold Strike (b203)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Ancestor (b203) — {3}{R}{W} 4/4 Spirit. Trample.
/// Aggressive body.
pub fn lorehold_ancestor_b203() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Ancestor (b203)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lorehold Spirit Squire (b203) — {W} 1/1 Spirit. ETB gain 2 life.
pub fn lorehold_spirit_squire_b203() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit Squire (b203)",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(2)],
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
    }
}
