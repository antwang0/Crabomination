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
    magecraft, magecraft_gain_life, magecraft_ping_any, magecraft_self_pump, target_filtered,
};
use crate::effect::{Duration, PlayerRef, ZoneDest};
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
    }
}

// ── Lorehold Apprentice ─────────────────────────────────────────────────────

/// Lorehold Apprentice — {R}{W}, 1/1 Human Cleric.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// you gain 1 life and Lorehold Apprentice deals 1 damage to any target."
///
/// Both halves of the magecraft rider wired: a `Seq` body of
/// `GainLife(1) + DealDamage(1)` against `target_filtered(Creature ∨
/// Player ∨ Planeswalker)`. The auto-target picker on triggers will
/// aim the 1 damage at any legal target (defaults to "an opponent"
/// for friendly-source pings); see `auto_target_for_effect_avoiding`
/// in the trigger registration path.
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
            // "Exile a card from your graveyard" — any card.
            exile_other_filter: Some(SelectionRequirement::Any),
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
            self_counter_cost_reduction: None,
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
            // Target picker walks every zone (incl. graveyards) when the
            // filter is `Any`, same as Ascendant Dustspeaker / Sundering
            // Archaic's "{2}: gy → bottom of library" target shape.
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
    use crate::card::CounterType;
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
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            },
            magecraft_self_pump(1, 0),
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
        triggered_abilities: vec![magecraft(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
                self_counter_cost_reduction: None,
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
                self_counter_cost_reduction: None,
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
                self_counter_cost_reduction: None,
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}
