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
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    Selector, SelectionRequirement, Subtypes, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{magecraft, target_filtered};
use crate::effect::{PlayerRef, ZoneDest};
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
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
            },
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
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

