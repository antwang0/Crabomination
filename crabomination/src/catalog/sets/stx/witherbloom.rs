//! Witherbloom (B/G) college cards from Strixhaven.
//!
//! Witherbloom's defining themes are Pest tokens (1/1 black-green creatures
//! that gain you 1 life when they die) and small-drain magecraft. This
//! module ships the Apprentice's drain trigger and a single Lesson token-
//! creator (Pest Summoning) at the simplified one-token level — see
//! STRIXHAVEN2.md for the death-trigger token TODO.

use super::no_abilities;
use super::shared::stx_pest_token;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Selector, SelectionRequirement, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::magecraft;
use crate::effect::{ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{cost, b, g, generic, Color, ManaCost};

// ── Witherbloom Apprentice ──────────────────────────────────────────────────

/// Witherbloom Apprentice — {B}{G}, 2/2 Human Warlock. "Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, each opponent
/// loses 1 life and you gain 1 life."
///
/// Wired via the new `EventSpec.filter` + `TriggerSource` binding. The
/// effect uses `Effect::Drain` over `EachOpponent`, which handles the
/// life-swap atomically.
pub fn witherbloom_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Apprentice",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
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
    }
}

// ── Pest Summoning (Lesson) ─────────────────────────────────────────────────

/// Pest Summoning — {B}{G} Sorcery — Lesson. Real Oracle creates two 1/1
/// black and green Pest tokens with "When this creature dies, you gain 1
/// life."
///
/// life."
///
/// Promoted to ✅: the token's "When this creature dies, you gain 1
/// life" trigger now rides on the new `TokenDefinition.triggered_abilities`
/// field. Each Pest dies → controller gains 1.
pub fn pest_summoning() -> CardDefinition {
    let pest = stx_pest_token();
    CardDefinition {
        name: "Pest Summoning",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        // Lesson is a sorcery sub-type. Add to the spell subtype list so
        // future Lesson-based mechanics (Mascot Exhibition, "search your
        // sideboard for a Lesson") can filter on it.
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        // Real Oracle creates *two* Pests; we now mint two, matching the
        // printed card. The token's "die → gain 1 life" trigger remains
        // ⏳ pending token-with-trigger plumbing — see STRIXHAVEN2.md.
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: pest,
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Bayou Groff ─────────────────────────────────────────────────────────────

/// Bayou Groff — {2}{B}{G}, 5/4 Beast. "When this creature dies, you
/// may pay {1}. If you do, return it to its owner's hand."
///
/// Now wired (push XVI) via the new `Effect::MayPay` primitive: dies
/// trigger asks the controller "Pay {1} to return Bayou Groff to your
/// hand?" — `AutoDecider` defaults to "no", `ScriptedDecider` can flip
/// to "yes" for tests. On "yes" + sufficient mana in pool, the engine
/// pays {1} and uses `Effect::Move(SelfSource → Hand(OwnerOf(Self)))`
/// to return the now-graveyard-resident card. The body resolves
/// against the just-died card by chasing its owner via
/// `PlayerRef::OwnerOf`.
pub fn bayou_groff() -> CardDefinition {
    CardDefinition {
        name: "Bayou Groff",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {1} to return Bayou Groff to your hand?".into(),
                mana_cost: ManaCost::new(vec![generic(1)]),
                body: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                }),
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

// ── Witherbloom Pledgemage ──────────────────────────────────────────────────

/// Witherbloom Pledgemage — {1}{B}{G}, 3/3 Plant Warlock. "{T}, Pay 1
/// life: Add {B} or {G}."
///
/// Push XXIV: promoted ✅. The "Pay 1 life" half of the cost is now
/// modeled with `ActivatedAbility.life_cost: 1` (push XV primitive),
/// matching the printed "as part of the activation cost" timing —
/// activation is rejected pre-pay with `GameError::InsufficientLife`
/// when life < 1, mirroring the mana-cost pre-pay check.
///
/// The "{B} or {G}" mode pick collapses to {B} for the auto-targeted
/// path; future modal-mana primitive can split the activation in two
/// (or thread a controller-decision into resolution). Net mana
/// generated is correct in either case.
pub fn witherbloom_pledgemage() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pledgemage",
        cost: cost(&[crate::mana::generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[]),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Black]),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 1,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Daemogoth Woe-Eater ─────────────────────────────────────────────────────

/// Daemogoth Woe-Eater — {2}{B}{G}, 9/9 Demon. "As an additional cost to
/// cast this spell, sacrifice a creature. / {T}: You gain 4 life."
///
/// 🟡 The "additional cost: sacrifice a creature" rider is approximated
/// as an ETB sacrifice trigger — the controller sacrifices another
/// creature when Woe-Eater enters. Same approach as Vicious Rivalry's
/// `Effect::LoseLife { XFromCost }` approximation of its "pay X life
/// additional cost" rider. The mana cost is the printed {2}{B}{G}; the
/// sac happens on ETB rather than at cast time, but the net board state
/// matches in all but a corner case (the spell can't be countered by
/// the sacrifice failing — rare in this engine).
pub fn daemogoth_woe_eater() -> CardDefinition {
    CardDefinition {
        name: "Daemogoth Woe-Eater",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 9,
        toughness: 9,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[]),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(4),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
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

// ── Eyeblight Cullers ───────────────────────────────────────────────────────

/// Eyeblight Cullers — {1}{B}{B}, 4/4 Elf Warrior.
/// "As an additional cost to cast this spell, sacrifice a creature. / When
/// this creature enters, target opponent loses 2 life and you gain 2 life."
///
/// 🟡 Same additional-cost approximation as Daemogoth Woe-Eater — the
/// sacrifice fires at ETB rather than at cast. The drain rider is wired
/// faithfully.
pub fn eyeblight_cullers() -> CardDefinition {
    CardDefinition {
        name: "Eyeblight Cullers",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
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
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
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
    }
}

// ── Dina, Soul Steeper ──────────────────────────────────────────────────────

/// Dina, Soul Steeper — {B}{G}, 1/3 Legendary Human Druid. "Deathtouch /
/// Whenever you gain life, target opponent loses 1 life. / {1}{B}{G}: Target
/// creature gets -X/-X until end of turn, where X is the number of
/// creatures you control."
///
/// Push XXX: 🟡 → ✅. The activated -X/-X is now properly scaled by
/// `Value::CountOf(EachPermanent(Creature ∧ ControlledByYou))` —
/// Dina counts as one of her own creatures (matches the printed
/// counting; the activation auto-targets a creature you don't control,
/// so the self-counting pump is never self-defeating). At a typical 3-
/// creature board state the activated ability shrinks the target by
/// 3/3 EOT (hard kill on most early-game blockers); at 5-creature
/// snowball it's -5/-5. Lifegain trigger remains wired against a target
/// opponent (auto-target picks any opponent).
///
/// Engine plumbing: `Value::Diff(Const(0), CountOf(...))` yields the
/// negated count (PumpPT accepts negative i32 power/toughness — same
/// shape as Lash of Malice's flat -2/-2). The new selector evaluates
/// at activation-resolution time, so casting Dina + ramping creatures
/// before activating snowballs the X.
pub fn dina_soul_steeper() -> CardDefinition {
    use crate::card::Keyword;
    use crate::effect::Duration;
    use crate::effect::shortcut::target_filtered;
    let creatures_you_control = Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    );
    let neg_x = Value::Diff(
        Box::new(Value::Const(0)),
        Box::new(Value::CountOf(Box::new(creatures_you_control))),
    );
    CardDefinition {
        name: "Dina, Soul Steeper",
        cost: cost(&[b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1), b(), g()]),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: neg_x.clone(),
                toughness: neg_x,
                duration: Duration::EndOfTurn,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
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

// ── Daemogoth Titan ─────────────────────────────────────────────────────────

/// Daemogoth Titan — {3}{B}{G}, 11/11 Demon Horror. Printed Oracle:
/// "Whenever this creature attacks or blocks, sacrifice another creature."
///
/// Push XXXI: ✅. The "or blocks" rider is now wired via the new
/// `EventKind::Blocks` event (push XXXI) — declare-blockers emits a
/// `BlockerDeclared` event and the Blocks/SelfSource trigger reads its
/// blocker side (filtered to *this* permanent), parallel to the existing
/// Attacks/SelfSource. Both halves run the same body — sacrifice a
/// non-titan creature you control. Combat-correct in every defender
/// scenario, not just attack-only swings.
pub fn daemogoth_titan() -> CardDefinition {
    let sac_another = Effect::Sacrifice {
        who: Selector::You,
        count: Value::Const(1),
        filter: SelectionRequirement::Creature
            .and(SelectionRequirement::ControlledByYou),
    };
    CardDefinition {
        name: "Daemogoth Titan",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon, CreatureType::Horror],
            ..Default::default()
        },
        power: 11,
        toughness: 11,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: sac_another.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: sac_another,
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

// ── Pest Infestation ────────────────────────────────────────────────────────

/// Pest Infestation — {X}{B}{G} Sorcery.
/// "Create X 1/1 black and green Pest creature tokens with 'When this
/// creature dies, you gain 1 life.'"
///
/// Push XXIV: ✅. Token count comes off the cast's X via `Value::XFromCost`
/// — same plumbing as Plumb the Forbidden / Pterafractyl. The minted Pest
/// token shares the `stx_pest_token()` definition, so its on-die lifegain
/// trigger rides on each token via `TokenDefinition.triggered_abilities`
/// (SOS push VI). At X=4 you mint four Pests for {4}{B}{G}; if any die
/// later, each fires its own +1-life trigger.
pub fn pest_infestation() -> CardDefinition {
    let pest = stx_pest_token();
    CardDefinition {
        name: "Pest Infestation",
        cost: cost(&[crate::mana::x(), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::XFromCost,
            definition: pest,
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Mortality Spear (Lesson) ────────────────────────────────────────────────

/// Mortality Spear — {3}{B}{G} Sorcery — Lesson.
/// "Destroy target creature or planeswalker."
///
/// Push XXX: ✅. Witherbloom's flexible Lesson removal — bigger fixed
/// cost than Necrotic Fumes ({1}{B}{B} sac-2 + exile-2) but no per-cast
/// rider, just a clean two-target-type kill. Wired with
/// `Effect::Destroy` on a `Creature OR Planeswalker` target filter (same
/// shape as Hero's Downfall / Killing Wave / Mage Hunters' Onslaught).
/// Lesson sub-type is set so future Learn-aware code can filter on it.
pub fn mortality_spear() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Mortality Spear",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
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
    }
}

// ── Witherbloom Command ─────────────────────────────────────────────────────

/// Witherbloom Command — {B}{G} Instant.
/// "Choose two —
/// • Target player loses 3 life and you gain 3 life.
/// • Return target permanent card with mana value 3 or less from your
///   graveyard to your hand.
/// • Destroy target enchantment.
/// • Target creature gets -3/-3 until end of turn."
///
/// Push XXXVI: ✅ — "choose two" now wires faithfully via the new
/// `Effect::ChooseModes { count: 2, up_to: false, allow_duplicates:
/// false }` primitive. Auto-decider picks modes 0+1 (drain 3 + gy →
/// hand). `ScriptedDecider::new([DecisionAnswer::Modes(vec![2, 3])])`
/// can pick destroy-enchantment + -3/-3 EOT for tests. Each individual
/// mode is wired faithfully against existing primitives:
/// - Mode 0: `Effect::Drain` for the 3 life swap (each-opponent-collapse).
/// - Mode 1: graveyard → hand on a permanent card with `ManaValueAtMost(3)`.
/// - Mode 2: destroy target enchantment.
/// - Mode 3: -3/-3 EOT pump.
pub fn witherbloom_command() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::shortcut::target_filtered;
    use crate::effect::{Duration, ZoneDest};
    let mv_at_most_3 = SelectionRequirement::Permanent
        .and(SelectionRequirement::ManaValueAtMost(3));
    CardDefinition {
        name: "Witherbloom Command",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseModes {
            count: 2,
            up_to: false,
            allow_duplicates: false,
            modes: vec![
                // Mode 0: drain 3.
                Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(3),
                },
                // Mode 1: gy → hand on permanent card MV ≤ 3.
                Effect::Move {
                    what: Selector::take(
                        Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: mv_at_most_3,
                        },
                        Value::Const(1),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                // Mode 2: destroy enchantment.
                Effect::Destroy {
                    what: target_filtered(SelectionRequirement::HasCardType(CardType::Enchantment)),
                },
                // Mode 3: -3/-3 EOT.
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(-3),
                    toughness: Value::Const(-3),
                    duration: Duration::EndOfTurn,
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
    }
}

// ── Foul Play ───────────────────────────────────────────────────────────────

/// Foul Play — {2}{B} Instant. Printed Oracle:
/// "Destroy target tapped creature.
///  If you control two or more Wizards, draw a card."
///
/// Push XXX: ✅. Witherbloom-flavoured tapped-creature kill. Wired via
/// `Effect::Seq([Destroy(Creature ∧ Tapped), If(≥2 Wizards, Draw 1)])`
/// — the existing `Predicate::ValueAtLeast(CountOf(...), 2)` shape
/// (same family as Galvanic Blast's metalcraft branching).
/// `Effect::If` resolves the gate against the controller's
/// battlefield Wizards-you-control count post-destroy. The Wizard
/// tribal predicate uses `HasCreatureType(Wizard)` so token Wizards
/// (Mascot Exhibition's birds aren't Wizards, but Spectacle Mage,
/// Symmetry Sage, Codespell Cleric all qualify) feed the gate.
pub fn foul_play() -> CardDefinition {
    use crate::card::{CreatureType, Predicate};
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Foul Play",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
                ),
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::count(Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::HasCreatureType(CreatureType::Wizard))
                            .and(SelectionRequirement::ControlledByYou),
                    )),
                    Value::Const(2),
                ),
                then: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
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
    }
}
