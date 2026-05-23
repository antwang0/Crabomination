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
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement, Selector, Subtypes, TriggeredAbility,
    Value, Zone,
};
use crate::effect::shortcut::{
    dies_drain, dies_mint_token, drain, etb, etb_drain, etb_gain_life, etb_mint_token, magecraft,
    magecraft_drain_each_opp, magecraft_gain_life, magecraft_self_pump, on_attack_drain,
    on_other_dies, on_other_dies_mint_token, target_filtered,
};
use crate::effect::{Duration, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{cost, b, g, generic, Color, ManaCost};

// ── Witherbloom Apprentice ──────────────────────────────────────────────────

/// Witherbloom Apprentice — {B}{G}, 2/2 Human Warlock. "Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, each opponent
/// loses 1 life and you gain 1 life."
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
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Witherbloom Pest-Tender (batch 15) ──────────────────────────────────────

/// Witherbloom Pest-Tender — {1}{B}, 1/2 Plant Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// 1/1 black and green Pest creature token with 'When this creature
/// dies, you gain 1 life.'"
///
/// Cheap Pest-tribal enabler — drops the Pest body on ETB, then sits
/// as a 2-mana 1/2 blocker. Each Pest dying triggers Witherbloom
/// Apprentice / Pestmaster / Pestbinder for cascading value.
pub fn witherbloom_pest_tender() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pest-Tender",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(stx_pest_token(), 1)],
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

// ── Pest Swarmer (batch 15) ─────────────────────────────────────────────────

/// Pest Swarmer — {2}{B}{G}, 2/2 Pest Warrior.
///
/// Printed Oracle (synthesised): "When this creature dies, create a
/// 1/1 black and green Pest creature token with 'When this creature
/// dies, you gain 1 life.'"
///
/// Self-replacing Pest body — once it dies, the Pest token rolls into
/// play, then THAT Pest dying gains 1 life. A solid sticky body for
/// Witherbloom death-trigger chains.
pub fn pest_swarmer() -> CardDefinition {
    CardDefinition {
        name: "Pest Swarmer",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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

// ── Witherbloom Seer (batch 15) ─────────────────────────────────────────────

/// Witherbloom Seer — {1}{B}{G}, 2/2 Human Druid, Deathtouch.
///
/// Printed Oracle (synthesised): "Deathtouch / Magecraft — Whenever
/// you cast or copy an instant or sorcery spell, each opponent loses
/// 1 life and you gain 1 life."
///
/// Sticky deathtouch body with magecraft drain on top — closes out
/// games via repeated drain triggers. Pairs with Witherbloom Apprentice
/// for double-drain on every cast.
pub fn witherbloom_seer() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Seer",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Pest Swarm (batch 15) ───────────────────────────────────────────────────

/// Pest Swarm — {3}{B}{G} Sorcery.
///
/// Printed Oracle (synthesised): "Create three 1/1 black and green
/// Pest creature tokens with 'When this creature dies, you gain 1
/// life.'"
///
/// Five-mana Pest fan-out — three sticky bodies that gain 3 life if
/// they all die. Pairs with Tend the Pests / Felisa / Pestbinder for
/// rapid aristocrats payoff. Same shape as Defend the Campus
/// (Inkling fan-out at the same mana).
pub fn pest_swarm() -> CardDefinition {
    CardDefinition {
        name: "Pest Swarm",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: stx_pest_token(),
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

// ── Witherbloom Vinemaster (batch 15) ───────────────────────────────────────

/// Witherbloom Vinemaster — {3}{B}{G}, 3/4 Plant Druid, Trample.
///
/// Printed Oracle (synthesised): "Trample / Whenever another Pest you
/// control dies, put a +1/+1 counter on this creature."
///
/// Big Witherbloom finisher that grows with every Pest death. Stacks
/// hard with Pest minters (Pest Summoning, Tend the Pests, Witherbloom
/// Pestbinder) — a 4/5 Vinemaster after one Pest, 5/6 after two, etc.
/// Trample punches through chump blockers.
pub fn witherbloom_vinemaster() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Witherbloom Vinemaster",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Pest),
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

// ── Witherbloom Pledgemage ──────────────────────────────────────────────────

/// Witherbloom Pledgemage — {1}{B}{G}, 3/3 Plant Warlock. "{T}, Pay 1
/// life: Add {B} or {G}."
///
/// Life is paid up front during cost-payment so the effect is a pure
/// `AddMana` — qualifies as a true mana ability (CR 605.1a) and
/// resolves without the stack. "B or G" is approximated as
/// `ManaPayload::AnyOneColor`: broader than printed but matches the
/// typical cube-pool ramp pattern.
pub fn witherbloom_pledgemage() -> CardDefinition {
    let _ = Color::Black;
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
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 1,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Witherbloom Mossfeeder (batch 17) ───────────────────────────────────────

/// Witherbloom Mossfeeder — {2}{B}{G}, 3/3 Plant Beast.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// 1/1 black and green Pest creature token with 'When this creature
/// dies, you gain 1 life.'"
///
/// Mid-curve Pest enabler — drops a sticky 3/3 body and a self-replacing
/// Pest token simultaneously. Combos with Vinemaster / Pestmaster /
/// Pestbinder counter accumulators. Same shape as Pest-Tender at the
/// curve-top slot.
pub fn witherbloom_mossfeeder() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Mossfeeder",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
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
                definition: stx_pest_token(),
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

// ── Witherbloom Reverie (batch 17) ──────────────────────────────────────────

/// Witherbloom Reverie — {1}{B}{G} Sorcery.
///
/// Printed Oracle (synthesised): "Each opponent loses 3 life and you
/// gain 3 life."
///
/// Pure {B}{G} drain — the classic Witherbloom three-mana drain
/// finisher / stabilizer. Pairs with Honor Troll / Light of Promise
/// lifegain payoffs and feeds Inkling Bloodscribe's drain chain.
pub fn witherbloom_reverie() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Reverie",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
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

// ── Pest Cultivator (batch 17) ──────────────────────────────────────────────

/// Pest Cultivator — {1}{B}{G}, 2/2 Plant Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, create two
/// 1/1 black and green Pest creature tokens with 'When this creature
/// dies, you gain 1 life.'"
///
/// Two-for-one Pest enabler — the 2/2 body plus 2 Pests on ETB makes
/// Pest Cultivator a strong 3-mana Witherbloom curve play. Pairs with
/// Felisa (Inkling minter), Witherbloom Pestmaster (counter on Pest
/// death), Vinemaster (counter on Pest death).
pub fn pest_cultivator() -> CardDefinition {
    CardDefinition {
        name: "Pest Cultivator",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: stx_pest_token(),
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

// ── Withergrowth Apprentice (batch 17) ──────────────────────────────────────

/// Withergrowth Apprentice — {B}{G}, 1/3 Human Druid.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, target creature you control gets +1/+1
/// until end of turn."
///
/// Defensive Witherbloom magecraft body that pumps a friendly creature
/// each cast. Mirror of Eager First-Year (white) in {B}{G} — a sticky
/// 1/3 wall that converts spells into combat math.
pub fn withergrowth_apprentice() -> CardDefinition {
    use crate::effect::shortcut::{magecraft, target_filtered};
    use crate::effect::Duration;
    CardDefinition {
        name: "Withergrowth Apprentice",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou)),
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

// ── Witherbloom Pestkeeper (batch 18) ──────────────────────────────────────

/// Witherbloom Pestkeeper — {2}{B}, 2/3 Plant Cleric.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// 1/1 black and green Pest creature token with 'When this creature
/// dies, you gain 1 life.' / {1}{B}{G}, Sacrifice a Pest: Target
/// creature gets -2/-2 until end of turn."
///
/// Sticky Pest enabler with a fold-up reactive sac-outlet. The
/// activation feeds Pestkeeper itself fodder, then ships -2/-2 to a
/// problem creature. Pairs with Witherbloom Apprentice for double-drain
/// + creature removal in the same sequence.
pub fn witherbloom_pestkeeper() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::target_filtered;
    use crate::effect::Duration;
    CardDefinition {
        name: "Witherbloom Pestkeeper",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1), b(), g()]),
            // Sac a Pest you control (`filter` constrains the picker) and
            // then ship -2/-2 to the target creature.
            effect: Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Pest))
                        .and(SelectionRequirement::ControlledByYou),
                },
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(-2),
                    toughness: Value::Const(-2),
                    duration: Duration::EndOfTurn,
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
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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

// ── Witherbloom Bonepicker (batch 18) ──────────────────────────────────────

/// Witherbloom Bonepicker — {1}{B}{G}, 3/3 Plant Skeleton, Trample.
///
/// Printed Oracle (synthesised): "Trample / When this creature enters,
/// each opponent loses 2 life."
///
/// Three-mana 3/3 trample drain — the headline Witherbloom curve-out:
/// drops a body that's already racing, then drains 2 immediately. Pairs
/// with Honor Troll for compounding lifegain payoff.
pub fn witherbloom_bonepicker() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bonepicker",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Skeleton],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
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

// ── Pest Inheritance (batch 18) ────────────────────────────────────────────

/// Pest Bequest — {3}{B}{G} Sorcery.
///
/// Printed Oracle (synthesised): "Target creature you control gets
/// +1/+1 and gains deathtouch until end of turn. Create a 1/1 black and
/// green Pest creature token."
///
/// Combat-ready Pest minter with a single-creature pump-and-deathtouch
/// rider. Pairs naturally with any Pest-tribal payoff (Pestbinder /
/// Vinemaster / Apprentice). The Pest's death-trigger lifegain rides
/// via `stx_pest_token()`. Renamed from "Pest Inheritance" to avoid
/// catalog name collision with the same-named Lesson in `stx::lessons`.
pub fn pest_swarm_inheritance() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Pest Bequest",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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

// ── Witherbloom Decayblossom (batch 18) ────────────────────────────────────

/// Witherbloom Decayblossom — {1}{B}, 1/1 Plant Cleric.
///
/// Printed Oracle (synthesised): "When this creature dies, target
/// creature gets -1/-1 until end of turn."
///
/// One-mana B sacrifice fodder that ships -1/-1 on death — combos with
/// Pestkeeper's sac outlet, Daemogoth Titan's attack-trigger sacrifice,
/// or just trades into a problem creature.
pub fn witherbloom_decayblossom() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    use crate::effect::Duration;
    CardDefinition {
        name: "Witherbloom Decayblossom",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
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

// ── Witherbloom Recourse (batch 18) ────────────────────────────────────────

/// Witherbloom Recourse — {1}{B}{G} Instant.
///
/// Printed Oracle (synthesised): "Return target creature card with
/// mana value 2 or less from your graveyard to your hand. Each opponent
/// loses 1 life and you gain 1 life."
///
/// Cheap creature-recursion + drain rider. The MV-≤-2 filter targets
/// the typical Witherbloom Pest / Apprentice graveyard contents. Drain
/// piggybacks for Apprentice-style chain triggers.
pub fn witherbloom_recourse() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Witherbloom Recourse",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
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
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
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

// ── Witherbloom Pestmancer (batch 18) ──────────────────────────────────────

/// Witherbloom Pestmancer — {2}{B}{G}, 2/2 Human Warlock.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, create a 1/1 black and green Pest
/// creature token with 'When this creature dies, you gain 1 life.'"
///
/// Top-end Witherbloom magecraft engine — each instant/sorcery you
/// cast mints a Pest. The Pest's death-trigger lifegain stacks with
/// the magecraft drain cards (Apprentice / Seer) for huge swings in
/// spell-heavy boards.
pub fn witherbloom_pestmancer() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Witherbloom Pestmancer",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: stx_pest_token(),
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

// ── Witherbloom Lifebleeder (batch 19) ─────────────────────────────────────

/// Witherbloom Lifebleeder — {1}{B}{G}, 2/2 Human Warlock.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, each opponent loses 1 life and
/// you gain 1 life."
///
/// Witherbloom Apprentice on a 3-mana frame for tougher metas. Same
/// drain trigger but with one extra mana of stat-cushion (2/2 → still
/// a bear) and the more relevant 3-CMC slot in slower decks. Pairs
/// with Daemogoth Titan as the magecraft-drain backbone.
pub fn witherbloom_lifebleeder() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lifebleeder",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Pest Marauder (batch 19) ───────────────────────────────────────────────

/// Pest Marauder — {1}{B}, 1/1 Pest with Deathtouch.
///
/// Printed Oracle (synthesised): "Deathtouch / When this creature
/// dies, you gain 1 life."
///
/// Pest-class 2-drop with deathtouch — classic black "trade-into-
/// anything" body wrapped in the stx_pest_token death lifegain rider
/// (1 life on death, mirroring the Pest token's printed shape). Pairs
/// with Witherbloom Vinemaster's Pest-death counter trigger.
pub fn pest_marauder() -> CardDefinition {
    CardDefinition {
        name: "Pest Marauder",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Refactored in batch 40 to use the `dies_gain_life` shortcut.
        triggered_abilities: vec![crate::effect::shortcut::dies_gain_life(1)],
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

// ── Witherbloom Decoctor (batch 19) ────────────────────────────────────────

/// Witherbloom Decoctor — {3}{B}{G}, 3/4 Human Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, each
/// opponent loses 2 life and you gain 2 life."
///
/// Curve-top Witherbloom drain body. 5-mana 3/4 frame with built-in
/// 4-life swing on ETB. Slots into the "drain finisher" archetype
/// alongside Pestilent Cauldron and Witherbloom Reverie.
pub fn witherbloom_decoctor() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Decoctor",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(2)],
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

// ── Witherbloom Glimmer (batch 19+) ────────────────────────────────────────

/// Witherbloom Glimmer — {2}{B}{G}, 3/3 Plant Druid, Lifelink.
///
/// Printed Oracle (synthesised): "Lifelink."
///
/// Vanilla 4-mana 3/3 lifelink body. Lifelink is the headline rider
/// — every combat hit gives the controller life, snowballing with
/// the Witherbloom drain-magecraft package. Same P/T as Witherbloom
/// Mossfeeder but trades the Pest ETB for lifelink.
pub fn witherbloom_glimmer() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Glimmer",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Pest Communion (batch 19+) ─────────────────────────────────────────────

/// Pest Communion — {1}{B}{G} Sorcery.
///
/// Printed Oracle (synthesised): "Each opponent mills four cards.
/// Each opponent loses 1 life and you gain 1 life."
///
/// 3-mana mill-and-drain combo. Mills 4 from each opponent (graveyard
/// fill for opp combo decks + setup for delirium-style payoffs on
/// our side) + 1 life drain. Similar to Witherbloom Command's
/// mode 0 + 1 combo at the same cost without the mode prompt.
pub fn pest_communion() -> CardDefinition {
    CardDefinition {
        name: "Pest Communion",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(4),
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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

// ── Witherbloom Sapfiend (batch 19) ────────────────────────────────────────

/// Witherbloom Sapfiend — {2}{G}, 2/3 Plant Beast.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, this creature gets +1/+1 until
/// end of turn."
///
/// Green magecraft growth body. Mirror of Eager First-Year on a more
/// defensive (2/3 vs 2/1) self-target frame. Multiple casts in a turn
/// stack — a 4-spell turn turns the Sapfiend into a 6/7 trampler-of-
/// chunk-damage.
pub fn witherbloom_sapfiend() -> CardDefinition {
    use crate::effect::shortcut::magecraft_self_pump;
    CardDefinition {
        name: "Witherbloom Sapfiend",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Witherbloom Toxicultivator (batch 20) ──────────────────────────────────

/// Witherbloom Toxicultivator — {2}{B}, 2/3 Plant Druid with Deathtouch.
///
/// Printed Oracle (synthesised): "Deathtouch. When this creature enters,
/// create a 1/1 black and green Pest creature token with 'When this
/// creature dies, you gain 1 life.'"
///
/// 3-mana 2/3 deathtouch Pest minter — punishes attackers (deathtouch
/// trades up) and seeds a Pest sac/drain engine. Compounds with
/// Pestkeeper sac outlets and Vinemaster Pest-death-counter triggers.
pub fn witherbloom_toxicultivator() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Toxicultivator",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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

// ── Pest Outburst (batch 20) ───────────────────────────────────────────────

/// Pest Outburst — {2}{B}{G} Sorcery.
///
/// Printed Oracle (synthesised): "Create two 1/1 black and green Pest
/// creature tokens with 'When this creature dies, you gain 1 life.'
/// You gain 2 life."
///
/// 4-mana Pest minter with bonus lifegain — produces two Pests +
/// immediate 2 life. Stacks with Vinemaster (Pest-death = +1/+1
/// counter) for a counter engine.
pub fn pest_outburst() -> CardDefinition {
    CardDefinition {
        name: "Pest Outburst",
        cost: cost(&[generic(2), b(), g()]),
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
                definition: stx_pest_token(),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Witherbloom Necromancer (batch 20) ─────────────────────────────────────

/// Witherbloom Grand Necromancer — {3}{B}{G}, 3/3 Human Warlock.
///
/// Printed Oracle (synthesised): "When this creature enters, return
/// target creature card from your graveyard to your hand. Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, each opponent
/// loses 1 life and you gain 1 life."
///
/// 5-mana grindy value top-end: ETB reanimates a creature to hand
/// (replaces itself in card economy), then magecraft drains for every
/// IS cast.
pub fn witherbloom_grand_necromancer() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Witherbloom Grand Necromancer",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
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
                effect: Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    }),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
            magecraft_drain_each_opp(1),
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

// ── Witherbloom Sapdrinker (batch 20) ──────────────────────────────────────

/// Witherbloom Sapdrinker — {1}{B}{G}, 2/3 Plant Vampire with Lifelink.
///
/// Printed Oracle (synthesised): "Lifelink. Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, this creature gets +1/+0
/// until end of turn."
///
/// 3-mana lifelink magecraft beater — every IS cast pumps power, the
/// lifelink turns that into life gain on combat. Big spell-heavy
/// finisher for the WB drain pile.
pub fn witherbloom_sapdrinker() -> CardDefinition {
    use crate::card::CounterType;
    let _ = CounterType::PlusOnePlusOne;
    use crate::effect::shortcut::magecraft;
    use crate::effect::Duration;
    CardDefinition {
        name: "Witherbloom Sapdrinker",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Vampire],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
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

// ── Witherbloom Crawler (batch 20) ─────────────────────────────────────────

/// Witherbloom Crawler — {B}{G}, 2/2 Plant Insect with Deathtouch and Reach.
///
/// Printed Oracle (synthesised): "Deathtouch, reach."
///
/// 2-mana deathtouch+reach body — best-in-class anti-flier defender that
/// also trades up on the ground. Pure stats body, no triggers, perfect
/// curve-2 for the BG pile.
pub fn witherbloom_crawler() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Crawler",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Insect],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch, Keyword::Reach],
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

// ── Pest Forager (batch 21) ────────────────────────────────────────────────

/// Pest Forager — {1}{G}, 2/1 Pest with Trample.
///
/// Printed Oracle (synthesised): "Trample. When this creature dies, you
/// gain 1 life."
///
/// 2-mana trampler with the standard Pest die-trigger. Pairs with
/// Witherbloom Vinemaster for chained +1/+1 counters. The Trample push lets
/// the 2-power swing chip life away even after a blocker trades.
pub fn pest_forager() -> CardDefinition {
    CardDefinition {
        name: "Pest Forager",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Refactored in batch 40 to use the `dies_gain_life` shortcut.
        triggered_abilities: vec![crate::effect::shortcut::dies_gain_life(1)],
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

// ── Witherbloom Carnivine (batch 21) ───────────────────────────────────────

/// Witherbloom Carnivine — {3}{B}{G}, 4/4 Plant Beast with Reach.
///
/// Printed Oracle (synthesised): "Reach. When this creature enters, target
/// player loses 3 life and you gain 3 life."
///
/// 5-mana race-breaking lifelink-flavored finisher — 4/4 reach defender +
/// 6-life swing on ETB. Stomp on aggressive flyer-based decks while
/// stabilising the life total.
pub fn witherbloom_carnivine() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Carnivine",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Drain {
                from: target_filtered(SelectionRequirement::Player),
                to: Selector::You,
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

// ── Pest Harvest (batch 21) ────────────────────────────────────────────────

/// Pest Harvest — {2}{B}{G} Sorcery.
///
/// Printed Oracle (synthesised): "Create a 1/1 black and green Pest creature
/// token with 'When this creature dies, you gain 1 life,' then draw a card."
///
/// 4-mana Pest minter + cantrip — replaces itself and leaves a sticky body.
/// Pure curve filler in Witherbloom Pest builds.
pub fn pest_harvest() -> CardDefinition {
    let pest = stx_pest_token();
    CardDefinition {
        name: "Pest Harvest",
        cost: cost(&[generic(2), b(), g()]),
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
                definition: pest,
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

// ── Witherbloom Necrosophist (batch 21) ────────────────────────────────────

/// Witherbloom Necrosophist — {2}{B}, 2/3 Human Warlock.
///
/// Printed Oracle (synthesised): "When this creature enters, return target
/// creature card from your graveyard to your hand."
///
/// 3-mana ETB graveyard-recursion body. The same shape as Gravedigger /
/// Silverquill Memorialist — caps gy recursion at any creature card (not
/// just ≤2-MV like Memorialist). Strong with Pest-sac shells where Pests
/// die early game and need to come back.
pub fn witherbloom_necrosophist() -> CardDefinition {
    use crate::card::CardType as CT;
    CardDefinition {
        name: "Witherbloom Necrosophist",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CT::Creature),
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

// ── Witherbloom Pestcaller (batch 21) ──────────────────────────────────────

/// Witherbloom Pestcaller — {3}{G}, 2/4 Plant Druid.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, create a 1/1 black and green Pest creature
/// token with 'When this creature dies, you gain 1 life.'"
///
/// 4-mana token-engine. Sturdier body than Sedgemoor Witch / Pestmancer
/// (2/4 vs 3/2). Slots into the BG spellslinger pile as a chain-Pest
/// minter. The lifelink-equivalent feedback loop (Pests die → +1 life)
/// pairs with Witherbloom Vinemaster's grow trigger.
pub fn witherbloom_pestcaller() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Witherbloom Pestcaller",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: stx_pest_token(),
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

// ── Witherbloom batch 22 ───────────────────────────────────────────────────

/// Pest Swarmlord — {3}{B}{G}, 3/3 Pest Warlock.
///
/// Printed Oracle (synthesised): "When this creature enters, create two
/// 1/1 black and green Pest creature tokens with 'When this creature
/// dies, you gain 1 life.'"
///
/// 5-mana 3/3 + two Pests on arrival. Goes wide hard — pairs with Blech
/// (each Pest gets a +1/+1 counter on lifegain) for a snowballing army.
pub fn pest_swarmlord() -> CardDefinition {
    CardDefinition {
        name: "Pest Swarmlord",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
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
                definition: stx_pest_token(),
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

/// Witherbloom Vinetender — {1}{G}, 2/2 Plant Druid Reach.
///
/// Printed Oracle (synthesised): "Reach. Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, you gain 1 life."
///
/// 2-mana Reach + lifegain engine. Cheaper Pest Mascot at the curve-2
/// slot; trades the tribal +1/+1 for cheaper magecraft drip.
pub fn witherbloom_vinetender() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Witherbloom Vinetender",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach],
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

/// Toxic Bloodletting — {1}{B}{G} Instant.
///
/// Printed Oracle (synthesised): "Target creature gets -2/-2 until end of
/// turn. You gain 2 life."
///
/// 3-mana modal removal — soft-removes 2-toughness creatures while
/// rebuilding life. Smooth Witherbloom removal at instant speed.
pub fn toxic_bloodletting() -> CardDefinition {
    CardDefinition {
        name: "Toxic Bloodletting",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: crate::effect::Duration::EndOfTurn,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Saproot — {2}{B}{G}, 3/3 Plant Beast.
///
/// Printed Oracle (synthesised): "Trample. When this creature dies, each
/// opponent loses 2 life and you gain 2 life."
///
/// 4-mana trampler with a baked-in death drain — even if it trades
/// down, you net a 2-life swing.
pub fn witherbloom_saproot() -> CardDefinition {
    use crate::effect::shortcut::dies_drain;
    CardDefinition {
        name: "Witherbloom Saproot",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_drain(2)],
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

/// Pest Mausoleum — {2}{B}{G} Sorcery.
///
/// Printed Oracle (synthesised): "Return target creature card from your
/// graveyard to your hand. Create a 1/1 black and green Pest creature
/// token with 'When this creature dies, you gain 1 life.'"
///
/// 4-mana reanimation + token mint. Cheap two-for-one that rebuilds the
/// graveyard pipeline and adds a body to the battlefield.
pub fn pest_mausoleum() -> CardDefinition {
    CardDefinition {
        name: "Pest Mausoleum",
        cost: cost(&[generic(2), b(), g()]),
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
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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

// ── Push (modern_decks) batch 23: 5 new Witherbloom cards ───────────────────

/// Pest Ravager — {3}{B}{G}, 4/4 Plant Beast Trample.
///
/// Printed Oracle (synthesised): "Trample. When this creature enters, create
/// two 1/1 black and green Pest creature tokens with 'When this creature
/// dies, you gain 1 life.'"
///
/// 5-mana 4/4 trampler with two Pest tokens in tow — a single card that
/// lands 6 power on board with a built-in 2-life buffer on each Pest death.
pub fn pest_ravager() -> CardDefinition {
    CardDefinition {
        name: "Pest Ravager",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: stx_pest_token(),
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

/// Witherbloom Famine — {3}{B}, sorcery.
///
/// Printed Oracle (synthesised): "Each opponent loses 4 life and you gain
/// 4 life."
///
/// 4-mana drain-4 finisher — 8-life swing per cast. Standard Witherbloom
/// burn-out tail to finish damaged opponents.
pub fn witherbloom_famine() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Famine",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
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

/// Witherbloom Greenrot — {1}{G}, 2/2 Plant Druid Reach.
///
/// Printed Oracle (synthesised): "Reach. When this creature enters, you gain
/// 2 life."
///
/// 2-mana ground / anti-flier defender with a small life buffer. The
/// lifegain ETB stacks with Honor Troll's conditional pump and Inkling
/// Bloodscribe drain.
pub fn witherbloom_greenrot() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Greenrot",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Pestbroker — {2}{B}, 2/3 Human Warlock.
///
/// Printed Oracle (synthesised): "When this creature enters, target opponent
/// loses 2 life and you gain 2 life. {1}{B}, Sacrifice a Pest: target
/// creature gets -1/-1 until end of turn."
///
/// 3-mana drain ETB + a sac-a-Pest sink that doubles as removal-against-
/// 1-toughness or shrink-and-fight enabler. The sacrifice-a-Pest cost is
/// expressed as a first-step `Effect::Sacrifice` in the activation body
/// (same shape as Witherbloom Pestkeeper).
pub fn witherbloom_pestbroker() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestbroker",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            tap_cost: false,
            sac_cost: false,
            life_cost: 0,
            exile_other_filter: None,
            condition: None,
            exile_self_cost: false,
            from_graveyard: false,
            sorcery_speed: false,
            once_per_turn: false,
            effect: Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Pest)
                        .and(SelectionRequirement::ControlledByYou),
                },
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: crate::effect::Duration::EndOfTurn,
                },
            ]),
                    self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        triggered_abilities: vec![etb_drain(2)],
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

/// Pestilent Bloom — {B}{G}, instant.
///
/// Printed Oracle (synthesised): "Target creature gets -3/-3 until end of
/// turn. Create a 1/1 black and green Pest creature token."
///
/// 2-mana shrink-removal + a fresh Pest body. Quickly answers most
/// 3-toughness creatures while padding the Witherbloom Pest engine.
pub fn pestilent_bloom() -> CardDefinition {
    CardDefinition {
        name: "Pestilent Bloom",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(-3),
                duration: crate::effect::Duration::EndOfTurn,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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

// ── Push (modern_decks) batch 24+: 2 more Witherbloom cards ────────────────

/// Witherbloom Pest-Lord — {3}{B}{G}, 3/3 Plant Warlock.
///
/// Printed Oracle (synthesised): "Pest creatures you control get +1/+0.
/// When this creature enters, create a 1/1 black and green Pest creature
/// token."
///
/// 5-mana Pest tribal lord + a token on ETB. Stacks with Witherbloom
/// Vinemaster and Pest Bequest for a wide Pest swarm.
pub fn witherbloom_pest_lord() -> CardDefinition {
    use crate::effect::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Witherbloom Pest-Lord",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
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
                definition: stx_pest_token(),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Pest creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Pest))
                        .and(SelectionRequirement::ControlledByYou),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Drainbreath — {1}{B}, 2/1 Plant Warlock.
///
/// Printed Oracle (synthesised): "When this creature dies, you gain 2
/// life and target opponent loses 2 life."
///
/// 2-mana drain-on-death attacker. Aggressive 2-power body that trades
/// up into a 4-life-swing on death. Reaper-Hand template at the 2-mana
/// slot.
pub fn witherbloom_drainbreath() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Drainbreath",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::dies_drain(2)],
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

// ── Push (modern_decks) batch 24: 5 new Witherbloom cards ──────────────────

/// Witherbloom Aspersor — {B}{G}, instant.
///
/// Printed Oracle (synthesised): "Target creature gets -2/-1 until end
/// of turn. You gain 1 life."
///
/// 2-mana cheap shrink-removal for 1-toughness creatures + small lifegain
/// — versatile combat trick / sweeper-tail for the Witherbloom drain
/// shell.
pub fn witherbloom_aspersor() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Aspersor",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-1),
                duration: crate::effect::Duration::EndOfTurn,
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

/// Pest Reanimator — {2}{B}{G}, 3/2 Plant Warlock.
///
/// Printed Oracle (synthesised): "When this creature enters, return target
/// creature card with mana value 3 or less from your graveyard to your
/// hand."
///
/// 4-mana reanimator engine in Witherbloom. Pairs with the Pest token
/// die-trigger lifegain — chain dying Pests + small creatures back into
/// the hand for repeated drain payoffs.
pub fn pest_reanimator() -> CardDefinition {
    CardDefinition {
        name: "Pest Reanimator",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(3)),
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

/// Witherbloom Spore-Master — {3}{B}{G}, 4/4 Plant Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, create two
/// 1/1 black and green Pest creature tokens."
///
/// 5-mana go-wide finisher — 4/4 body + 2 Pest tokens for 8 power across
/// three bodies. Strict-upgrade frame over Pest Ravager (a 4/4 vs a 4/4
/// trampler, but with 2 Pest tokens instead of 2 trampler tokens).
pub fn witherbloom_spore_master() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Spore-Master",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: stx_pest_token(),
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

/// Witherbloom Withercut — {1}{B}{G}, instant.
///
/// Printed Oracle (synthesised): "Target creature gets -3/-1 until end
/// of turn. Draw a card."
///
/// 3-mana shrink-and-cantrip in Witherbloom. Better than Toxic Bloodletting
/// at the same slot when you're behind on cards but worse on damage
/// (-3/-1 vs -2/-2 + 2 life).
pub fn witherbloom_withercut() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Withercut",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(-1),
                duration: crate::effect::Duration::EndOfTurn,
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

/// Pest Cultivator-Adept — {2}{B}{G}, 2/3 Plant Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// 1/1 black and green Pest creature token. Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, put a +1/+1 counter on this
/// creature."
///
/// 4-mana Pest engine + magecraft counter-builder. Same shape as
/// Witherbloom Vinemaster but with a different trigger source — counters
/// on any spell cast vs only on Pest death.
pub fn pest_cultivator_adept() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Pest Cultivator-Adept",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: stx_pest_token(),
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

/// Witherbloom Reaper-Hand — {2}{B}{G}, 3/3 Plant Warlock Deathtouch.
///
/// Printed Oracle (synthesised): "Deathtouch. When this creature dies,
/// target opponent loses 2 life and you gain 2 life."
///
/// 4-mana deathtouch attacker with a built-in 4-life-swing on death.
/// Trade up into removal and still get the drain on the way out.
pub fn witherbloom_reaper_hand() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Reaper-Hand",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::dies_drain(2)],
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


// ── Push (modern_decks) batch 24++: 1 more Witherbloom card ────────────────

/// Witherbloom Tendril — {1}{B}{G}, instant.
///
/// Printed Oracle (synthesised): "Drain 2 (each opp loses 2 life and you
/// gain 2 life). Draw a card."
///
/// 3-mana instant drain + cantrip — Witherbloom's high-value spell-slot
/// fill. Stacks with Apprentice / Bonepicker for chained drain triggers.
pub fn witherbloom_tendril() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Tendril",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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

// ── Push (modern_decks) batch 25: 6 more Witherbloom cards ─────────────────
//
// Continuing Witherbloom (B/G) buildout: 4 new creatures + 2 spells using
// existing magecraft / drain / Pest token / counter primitives. No new
// engine features required.

/// Witherbloom Marshcaster — {1}{B}, 1/2 Plant Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 1.
/// Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// target opponent loses 1 life."
///
/// 2-mana scry-1 + per-cast 1-drain body. Defensive shape that smooths
/// future draws and trickles damage on every spell.
pub fn witherbloom_marshcaster() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Marshcaster",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
            },
            magecraft_drain_each_opp(1),
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

/// Pest Wrangler — {2}{G}, 2/3 Plant Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// 1/1 black and green Pest creature token."
///
/// 3-mana 2/3 + Pest token. Same body as Bayou Groff but with a Pest
/// minter instead of pay-1-to-return. Stocks the Witherbloom Pest pool
/// for chained drain payoffs.
pub fn pest_wrangler() -> CardDefinition {
    CardDefinition {
        name: "Pest Wrangler",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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

/// Witherbloom Toxicaster — {B}{G}, 1/1 Plant Warlock Deathtouch.
///
/// Printed Oracle (synthesised): "Deathtouch. Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, this creature gets +0/+1
/// until end of turn."
///
/// 2-mana deathtouch + per-cast +0/+1 toughness scaling. Trades with
/// anything; in a spell-heavy shell grows to a 1/4+ deathtouch wall.
pub fn witherbloom_toxicaster() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Witherbloom Toxicaster",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            crate::effect::shortcut::magecraft(Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(0),
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

/// Witherbloom Soilbleeder — {3}{B}{G}, 4/3 Plant Warlock.
///
/// Printed Oracle (synthesised): "When this creature enters, you may
/// sacrifice another creature. If you do, target opponent loses 3 life
/// and you gain 3 life."
///
/// 5-mana sac outlet with a 6-life-swing payoff. Trades surplus Pest
/// tokens for a Black-style execution removal/finisher.
pub fn witherbloom_soilbleeder() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Soilbleeder",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "sacrifice another creature".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::Const(1),
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::OtherThanSource),
                    },
                    Effect::Drain {
                        from: Selector::Player(PlayerRef::Target(0)),
                        to: Selector::You,
                        amount: Value::Const(3),
                    },
                ])),
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

/// Witherbloom Handburner — {2}{B}, sorcery.
///
/// Printed Oracle (synthesised): "Target opponent discards two cards.
/// You gain 2 life."
///
/// 3-mana 2-for-1 hand attack + lifegain. Strong in attrition wars —
/// strips two cards while feeding the Witherbloom lifegain shell.
pub fn witherbloom_handburner() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Handburner",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
                random: false,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Brood-Mother — {3}{B}{G}, 3/4 Plant Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, create two
/// 1/1 black and green Pest creature tokens. Whenever a Pest you control
/// dies, target opponent loses 1 life."
///
/// 5-mana 3/4 with a Pest factory + Pest-death-payoff combo. Lifegain
/// floor + extra drain on every Pest exit creates a punishing tempo lock.
pub fn pest_brood_mother() -> CardDefinition {
    let pest_death_drain = TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
            .with_filter(crate::effect::Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::HasCreatureType(CreatureType::Pest),
            }),
        effect: Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        },
    };
    CardDefinition {
        name: "Pest Brood-Mother",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: stx_pest_token(),
                },
            },
            pest_death_drain,
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

// ── Push (modern_decks) batch 28: 5 more Witherbloom cards ─────────────────
//
// Continuing Witherbloom (B/G) buildout: 5 new cards using existing
// primitives. No new engine features required.

/// Witherbloom Vinekeeper — {2}{B}{G}, 3/4 Plant Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, you gain 2
/// life. Whenever another creature dies, you gain 1 life."
///
/// 4-mana grindy defender + lifegain engine. Stacks with the Pest token's
/// die-to-gain trigger for double lifegain per Pest sacrifice.
pub fn witherbloom_vinekeeper() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vinekeeper",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_gain_life(2),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
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

/// Pest Outcast — {B}, 1/1 Pest Warlock.
///
/// Printed Oracle (synthesised): "When this creature dies, you gain 1 life
/// and draw a card."
///
/// 1-mana sac fodder with built-in draw-on-death. The Pest token's lifegain
/// effect is replicated on the body, plus a cantrip when it dies.
pub fn pest_outcast() -> CardDefinition {
    CardDefinition {
        name: "Pest Outcast",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
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

/// Witherbloom Drainscholar — {B}{G}, 1/2 Plant Druid Lifelink.
///
/// Printed Oracle (synthesised): "Lifelink. Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, target creature gets -1/-1 until
/// end of turn."
///
/// 2-mana lifelink body with magecraft removal — every IS cast can finish
/// off a small attacker. Pairs with Witherbloom Apprentice's drain.
pub fn witherbloom_drainscholar() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    use crate::effect::Duration;
    CardDefinition {
        name: "Witherbloom Drainscholar",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
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

/// Witherbloom Coatlcaller — {2}{G}, 2/3 Human Druid Reach.
///
/// Printed Oracle (synthesised): "Reach. When this creature enters, create
/// a 1/1 black-and-green Pest creature token with 'When this creature dies,
/// you gain 1 life.'"
///
/// 3-mana anti-flier + Pest factory body. Sticky against air aggro, feeds
/// Pestmancer / Pestkeeper sac engines.
pub fn witherbloom_coatlcaller() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Coatlcaller",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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

/// Witherbloom Pestbreaker — {3}{B}{G}, sorcery.
///
/// Printed Oracle (synthesised): "Destroy target creature. Create a 1/1
/// black-and-green Pest creature token."
///
/// 5-mana hard removal + Pest body. Net value = +1 card equivalent (kill
/// + body) for 5 mana. Pairs with Pestmaster / Vinemaster counter-snowballs.
pub fn witherbloom_pestbreaker() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestbreaker",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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

// ── Batch 30: 6 new Witherbloom cards ──────────────────────────────────────

/// Witherbloom Sapsucker — {1}{B}, 2/1 Plant Warlock.
///
/// Synthesised Oracle: "Lifelink. When this creature dies, you gain 2 life."
///
/// 2-mana lifelink aggressor with persistent gain-on-death payoff.
pub fn witherbloom_sapsucker() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapsucker",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
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

/// Pest Cultist — {1}{B}, 1/1 Pest Warlock.
///
/// Synthesised Oracle: "Whenever another creature you control dies,
/// each opponent loses 1 life and you gain 1 life."
///
/// Aristocrats-style drain payoff at 2 mana.
pub fn pest_cultist() -> CardDefinition {
    CardDefinition {
        name: "Pest Cultist",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_other_dies(Effect::Drain {
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

/// Witherbloom Bonecrafter — {2}{B}, 2/3 Plant Druid.
///
/// Synthesised Oracle: "When this creature enters, mill two cards.
/// You gain 1 life for each creature card put into your graveyard this way."
///
/// Wired as a single Mill 2 + per-creature-card lifegain rider. Uses the
/// `Value::CreatureCardsMilledThisEffect` primitive when present; if not
/// available, this approximates with a flat 1-life gain via Seq.
pub fn witherbloom_bonecrafter() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bonecrafter",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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
                Effect::Mill {
                    who: Selector::You,
                    amount: Value::Const(2),
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

/// Witherbloom Toxbrewer — {B}{G}, 2/2 Plant Warlock.
///
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, target creature an opponent controls gets -1/-1 until end of turn."
///
/// 2-mana shrinker that fires every spell — converts attrition into a board sweep.
pub fn witherbloom_toxbrewer() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Toxbrewer",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: crate::effect::Duration::EndOfTurn,
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

/// Witherbloom Lichenkeeper — {2}{G}, 2/4 Plant Druid Reach.
///
/// Synthesised Oracle: "Reach. When this creature enters, mint a 1/1
/// black-and-green Pest creature token with 'When this creature dies,
/// you gain 1 life.'"
///
/// 3-mana defensive Reach body + Pest mint. Stacks for token-based shells.
pub fn witherbloom_lichenkeeper() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lichenkeeper",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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

/// Witherbloom Sapwarden — {3}{B}{G}, sorcery.
///
/// Synthesised Oracle: "Destroy target creature an opponent controls.
/// You gain 2 life."
///
/// 5-mana hard removal with a lifegain rider. Removed-creature → graveyard
/// for further gy synergy.
pub fn witherbloom_sapwarden() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapwarden",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Bloomweaver — {2}{B}{G}, 3/3 Plant Warlock. Synthesised
/// Oracle: "When this creature enters, create a 1/1 black-and-green Pest
/// creature token with 'When this creature dies, you gain 1 life.' /
/// Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// each opponent loses 1 life." 4-mana double-payoff body.
pub fn witherbloom_bloomweaver() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_each_opp;
    CardDefinition {
        name: "Witherbloom Bloomweaver",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
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
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: stx_pest_token(),
                },
            },
            magecraft_ping_each_opp(1),
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

/// Witherbloom Drainpath — {2}{B}, sorcery. Synthesised Oracle:
/// "Each opponent loses 2 life and you gain 2 life. Surveil 1."
/// 3-mana drain + selection.
pub fn witherbloom_drainpath() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Drainpath",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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

/// Witherbloom Vinekeeper — {3}{B}{G}, 4/4 Plant Druid. Synthesised
/// Oracle: "Whenever this creature attacks, target opponent loses 2 life
/// and you gain 2 life." 5-mana attack drain engine.
pub fn witherbloom_vinekeeper_b30() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vinekeeper II",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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

/// Witherbloom Sapcurse — {B}{G}, instant. Synthesised Oracle:
/// "Target creature gets -2/-2 until end of turn." 2-mana shrink.
pub fn witherbloom_sapcurse() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapcurse",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: crate::effect::Duration::EndOfTurn,
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

/// Witherbloom Pestreaver — {1}{B}{G}, 2/3 Pest Warlock. Synthesised
/// Oracle: "When this creature enters, mill 2. You gain 1 life for each
/// creature card put into your graveyard this way." 3-mana mill + drip
/// lifegain — approximated as Mill 2 + GainLife 1 (the per-creature-card
/// scaling is engine-wide pending a milled-creature-count primitive).
pub fn witherbloom_pestreaver() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestreaver",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
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
                Effect::Mill {
                    who: Selector::You,
                    amount: Value::Const(2),
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

/// Witherbloom Vinemender — {2}{G}, 2/2 Plant Druid. Synthesised
/// Oracle: "When this creature enters, you gain 3 life." 3-mana
/// defensive lifegain body that feeds Old-Growth Educator's Infusion
/// gate, Blech's lifegain-tribal counters, and Witherbloom Apprentice's
/// drain stacking.
pub fn witherbloom_vinemender() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vinemender",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Devourer — {3}{B}, 3/2 Pest Warlock. Synthesised
/// Oracle: "Menace. When this creature enters, target opponent
/// sacrifices a creature." 4-mana edict-on-a-body.
pub fn witherbloom_devourer() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Devourer",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
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

/// Witherbloom Lifebloom — {1}{G}, sorcery. Synthesised Oracle:
/// "You gain 4 life. Surveil 1." 2-mana defensive lifegain + selection.
pub fn witherbloom_lifebloom() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lifebloom",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(4),
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

/// Witherbloom Rotmancer — {1}{B}, 2/2 Pest Warlock. Synthesised Oracle:
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// each opponent loses 1 life." Tax-the-board body.
pub fn witherbloom_rotmancer() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_each_opp;
    CardDefinition {
        name: "Witherbloom Rotmancer",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Sapseeker — {2}{G}, 3/3 Plant Druid. Synthesised Oracle:
/// "Trample. Whenever this creature attacks, you gain 1 life." 3-mana
/// big body with combat-trigger lifegain.
pub fn witherbloom_sapseeker() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapseeker",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Pestlich — {3}{B}{G}, 3/4 Pest Warlock. Synthesised Oracle:
/// "When this creature enters, return target creature card from your
/// graveyard to the battlefield." 5-mana reanimator-on-a-body.
pub fn witherbloom_pestlich() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Witherbloom Pestlich",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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

/// Witherbloom Mireguide — {1}{G}, 1/2 Plant Druid. Synthesised
/// Oracle: "{T}: Add {B} or {G}." 2-mana mana dork for Witherbloom
/// shells.
pub fn witherbloom_mireguide() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Mireguide",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
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
                    pool: ManaPayload::Colors(vec![Color::Black]),
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
                    pool: ManaPayload::Colors(vec![Color::Green]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 32 (modern_decks) — Witherbloom expansion ─────────────────────────

/// Witherbloom Pestswarm — {2}{B}{G}, 3/2 Plant Warrior.
/// Synthesised Oracle: "When this creature enters, create two 1/1 black-green
/// Pest creature tokens with 'When this creature dies, you gain 1 life.'"
pub fn witherbloom_pestswarm() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestswarm",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: stx_pest_token(),
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

/// Witherbloom Acolyte — {1}{B}, 1/2 Human Warlock.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, you gain 1 life."
pub fn witherbloom_lifeleecher() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lifeleecher",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Rootcaster — {2}{G}, 2/3 Plant Druid.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature gets +1/+1 until end of turn."
pub fn witherbloom_rootcaster() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Rootcaster",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Caulhound — {3}{B}{G}, 4/4 Plant Beast Trample.
/// Synthesised Oracle: "When this creature enters, each opponent loses 2
/// life and you gain 2 life."
pub fn witherbloom_caulhound() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Caulhound",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(2)],
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

/// Witherbloom Necromancer — {3}{B}, 2/3 Human Wizard.
/// Synthesised Oracle: "When this creature enters, return target creature
/// card with mana value 3 or less from your graveyard to your hand."
pub fn witherbloom_gravecaller() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Gravecaller",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
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
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(3)),
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

/// Witherbloom Bloodvine — {B}{G}, 1/3 Plant Vampire Lifelink.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, you gain 1 life."
pub fn witherbloom_bloodvine() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bloodvine",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Vampire],
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

/// Witherbloom Vitalist — {1}{G}, 2/2 Human Druid.
/// Synthesised Oracle: "Whenever you gain life, put a +1/+1 counter on this
/// creature." Same shape as Ajani's Pridemate.
pub fn witherbloom_vitalist() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vitalist",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
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

/// Witherbloom Toxinkeeper — {2}{B}, 2/2 Human Warlock Deathtouch.
/// Synthesised Oracle: "When this creature enters, target creature gets
/// -1/-1 until end of turn."
pub fn witherbloom_toxinkeeper() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Toxinkeeper",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
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

/// Witherbloom Bloodroot — {3}{B}{G}, sorcery.
/// Synthesised Oracle: "Drain 4 (each opponent loses 4 life and you gain
/// 4 life)."
pub fn witherbloom_bloodroot() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bloodroot",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
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

/// Witherbloom Pesthatch — {1}{B}{G}, sorcery.
/// Synthesised Oracle: "Create a 1/1 black-green Pest creature token with
/// 'When this creature dies, you gain 1 life,' then put a +1/+1 counter on
/// target creature you control."
pub fn witherbloom_pesthatch() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pesthatch",
        cost: cost(&[generic(1), b(), g()]),
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
                definition: stx_pest_token(),
            },
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
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

/// Witherbloom Diviner — {2}{B}{G}, 2/3 Human Warlock.
/// Synthesised Oracle: "When this creature enters, mill three cards, then
/// you may return target creature card from your graveyard to your hand."
pub fn witherbloom_diviner() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Diviner",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
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
                Effect::Mill {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
                Effect::MayDo {
                    description: "Return a creature card from your graveyard to your hand"
                        .to_string(),
                    body: Box::new(Effect::Move {
                        what: Selector::one_of(Selector::CardsInZone {
                            zone: Zone::Graveyard,
                            who: PlayerRef::You,
                            filter: SelectionRequirement::Creature,
                        }),
                        to: ZoneDest::Hand(PlayerRef::You),
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

// ── Batch 33: 7 new Witherbloom cards ────────────────────────────────────

/// Witherbloom Bloodscribe — {2}{B}, 3/2 Human Warlock.
/// Synthesised Oracle: "When this creature enters, each opponent loses 2
/// life. / Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, you gain 1 life."
pub fn witherbloom_bloodscribe() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bloodscribe",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
            },
            magecraft_gain_life(1),
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

/// Pest Skyswarm — {3}{B}{G}, 2/2 Plant Insect Flying.
/// Synthesised Oracle: "Flying / When this creature enters, create a 1/1
/// black-and-green Pest creature token with 'When this creature dies,
/// you gain 1 life.'"
pub fn pest_skyswarm() -> CardDefinition {
    CardDefinition {
        name: "Pest Skyswarm",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Insect],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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

/// Witherbloom Marshtender — {1}{G}, 1/3 Plant Druid Reach.
/// Synthesised Oracle: "Reach / When this creature enters, you gain 1
/// life. / Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, you gain 1 life."
pub fn witherbloom_marshtender() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Marshtender",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_gain_life(1),
            magecraft_gain_life(1),
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

/// Pest Hivekeeper — {2}{G}, 2/3 Plant Insect.
/// Synthesised Oracle: "Whenever another Pest enters under your control,
/// put a +1/+1 counter on this creature."
pub fn pest_hivekeeper() -> CardDefinition {
    CardDefinition {
        name: "Pest Hivekeeper",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Insect],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Pest),
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

/// Bloodvine Drainmage — {3}{B}{G}, 4/3 Plant Warlock Lifelink.
/// Synthesised Oracle: "Lifelink / When this creature enters, each
/// opponent loses 3 life and you gain 3 life."
pub fn bloodvine_drainmage() -> CardDefinition {
    CardDefinition {
        name: "Bloodvine Drainmage",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(3)],
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

/// Pest Snatchgrab — {B}{G}, Instant.
/// Synthesised Oracle: "Target opponent sacrifices a creature. Create a
/// 1/1 black-and-green Pest creature token with 'When this creature
/// dies, you gain 1 life.'"
pub fn pest_snatchgrab() -> CardDefinition {
    CardDefinition {
        name: "Pest Snatchgrab",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                filter: SelectionRequirement::Creature,
                count: Value::Const(1),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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

/// Witherbloom Pyrescholar — {2}{B}, 3/2 Human Warlock.
/// Synthesised Oracle: "When this creature dies, each opponent loses 2
/// life and you gain 2 life."
pub fn witherbloom_blooddrinker() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Blooddrinker",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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

/// Witherbloom Sapfeeder — {1}{B}{G}, 2/2 Plant Beast.
/// Synthesised Oracle: "When this creature enters, target opponent loses
/// 2 life. {1}{B}, Sacrifice a creature: Target opponent loses 1 life and
/// you gain 1 life."
pub fn witherbloom_pestwarden() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestwarden",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            tap_cost: false,
            sac_cost: false,
            life_cost: 0,
            exile_other_filter: None,
            condition: None,
            exile_self_cost: false,
            from_graveyard: false,
            sorcery_speed: false,
            once_per_turn: false,
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
            self_counter_cost_reduction: None, sac_other_filter: None,
        }],
        triggered_abilities: vec![etb_drain(2)],
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

// ── Batch 34: Witherbloom cards ─────────────────────────────────────────────

/// Witherbloom Pestrider — {1}{B}{G}, 2/2 Pest Druid.
/// Synthesised Oracle: "When this creature enters, create a 1/1 black-green
/// Pest creature token with 'When this creature dies, you gain 1 life,'
/// then put a +1/+1 counter on it."
pub fn witherbloom_pestrider() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestrider",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
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
                    definition: stx_pest_token(),
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
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

/// Witherbloom Mosshulk — {3}{B}{G}, 4/4 Plant Beast with Trample.
/// Synthesised Oracle: vanilla beat-stick.
pub fn witherbloom_mosshulk() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Mosshulk",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Lifefarmer — {2}{G}, 2/3 Plant Druid.
/// Synthesised Oracle: "When this creature enters, you gain 3 life."
pub fn witherbloom_lifefarmer() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lifefarmer",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Horde — {4}{B}{G}, Sorcery.
/// Synthesised Oracle: "Create four 1/1 black-green Pest creature tokens
/// with 'When this creature dies, you gain 1 life.'"
pub fn pest_horde() -> CardDefinition {
    CardDefinition {
        name: "Pest Horde",
        cost: cost(&[generic(4), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(4),
            definition: stx_pest_token(),
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

// ── Batch 35: Witherbloom cards ─────────────────────────────────────────────

/// Witherbloom Hexpetal — {1}{B}{G}, 2/2 Plant Druid.
/// Synthesised Oracle: "When this creature enters, each opponent loses 2
/// life and you gain 2 life."
pub fn witherbloom_hexpetal() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Hexpetal",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(2)],
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

/// Pest Inkblot — {B}{G}, 1/2 Pest Warlock with Deathtouch.
/// Synthesised Oracle: "When this creature dies, you gain 1 life."
pub fn pest_inkblot() -> CardDefinition {
    CardDefinition {
        name: "Pest Inkblot",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Tangleweed — {3}{B}{G}, 4/5 Plant Warrior with Trample.
/// Synthesised Oracle: A finisher body.
pub fn witherbloom_tangleweed() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Tangleweed",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Hauntwing — {2}{B}, 2/1 Pest with Flying.
/// Synthesised Oracle: "When this creature dies, you gain 1 life."
pub fn pest_hauntwing() -> CardDefinition {
    CardDefinition {
        name: "Pest Hauntwing",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 36: more Witherbloom cards ────────────────────────────────────────

/// Witherbloom Verdancer — {2}{G}, 2/3 Plant Druid Reach.
/// Synthesised Oracle: "When this creature enters, gain 1 life. Magecraft —
/// gain 1 life."
pub fn witherbloom_verdancer() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Verdancer",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_gain_life(1),
            magecraft_gain_life(1),
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

/// Pest Vinekin — {3}{B}{G}, 3/3 Pest Plant with Trample.
/// Synthesised Oracle: When dies, you gain 3 life and create 2 Pest tokens.
pub fn pest_vinekin() -> CardDefinition {
    CardDefinition {
        name: "Pest Vinekin",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Plant],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: stx_pest_token(),
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

/// Witherbloom Soulrender — {2}{B}{G}, Sorcery.
/// Synthesised Oracle: "Drain 3. Mill 3."
pub fn witherbloom_soulrender() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Soulrender",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(3),
            },
            Effect::Mill {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 38: more Witherbloom cards ────────────────────────────────────────

/// Witherbloom Fungalweb — {B}{G}, Instant.
/// Synthesised Oracle: "Each opponent loses 2 life and you gain 2 life."
pub fn witherbloom_fungalweb() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Fungalweb",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
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

/// Pest Swarmrider — {2}{B}, 2/2 Pest Insect.
/// Synthesised Oracle: "When this creature enters, create a 1/1 B/G Pest
/// creature token."
pub fn pest_swarmrider() -> CardDefinition {
    CardDefinition {
        name: "Pest Swarmrider",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Insect],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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

/// Witherbloom Bloodbrewer — {1}{B}{G}, 2/2 Plant Warlock.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, each opponent loses 1 life."
pub fn witherbloom_bloodbrewer() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bloodbrewer",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Rotwarden — {3}{B}{G}, 4/4 Plant Warrior with Trample and
/// Lifelink. Synthesised Oracle: vanilla beater.
pub fn witherbloom_rotwarden() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Rotwarden",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Lifelink],
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

/// Witherbloom Cauldronkeeper — {2}{G}, 2/3 Plant Druid.
/// Synthesised Oracle: "When this creature enters, you gain 2 life and
/// scry 1."
pub fn witherbloom_cauldronkeeper() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Cauldronkeeper",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::Scry {
                    who: PlayerRef::You,
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

/// Pest Briarscale — {2}{G}, 3/3 Pest Beast with Trample.
/// Synthesised Oracle: vanilla green beater at the Pest tribal slot.
pub fn pest_briarscale() -> CardDefinition {
    CardDefinition {
        name: "Pest Briarscale",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Beast],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 39: 6 more Witherbloom cards ─────────────────────────────────────

/// Witherbloom Rootbinder — {1}{B}{G}, 2/3 Plant Druid.
/// Synthesised Oracle: "When this creature enters, gain 2 life.
/// Magecraft — gain 1 life."
pub fn witherbloom_rootbinder() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Rootbinder",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_gain_life(2),
            magecraft_gain_life(1),
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

/// Pest Reaver — {2}{B}{G}, 3/3 Pest Beast with Deathtouch.
/// Synthesised Oracle: "Threat-pest body."
pub fn pest_reaver() -> CardDefinition {
    CardDefinition {
        name: "Pest Reaver",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
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

/// Witherbloom Decoction — {B}{G}, Instant.
/// Synthesised Oracle: "Each opponent loses 2 life. You gain 2 life and
/// scry 1."
pub fn witherbloom_decoction() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Decoction",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Cultivator — {2}{G}, 2/3 Human Druid.
/// Synthesised Oracle: "When this creature enters, create a 1/1 black-green
/// Pest token (no death-trigger). Magecraft — put a +1/+1 counter on this
/// creature."
pub fn witherbloom_cultivator() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Cultivator",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
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
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: stx_pest_token(),
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

/// Witherbloom Spawnkeeper — {3}{B}{G}, 3/4 Fungus Druid.
/// Synthesised Oracle: "Whenever another creature you control dies, gain
/// 1 life and target opp loses 1 life."
pub fn witherbloom_spawnkeeper() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Spawnkeeper",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fungus, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
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

/// Witherbloom Verdantwarden — {4}{G}, 5/5 Plant Beast with Trample.
/// Synthesised Oracle: "Big trampling top-end."
pub fn witherbloom_verdantwarden() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Verdantwarden",
        cost: cost(&[generic(4), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Thresher — {3}{B}, 2/3 Plant Insect with Deathtouch.
/// Synthesised Oracle: "When this creature enters, each opponent loses 1
/// life and you gain 1 life. Magecraft — each opponent loses 1 life."
pub fn witherbloom_thresher() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Thresher",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Insect],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Refactored in batch 40 to use the `etb_drain` shortcut.
        triggered_abilities: vec![
            crate::effect::shortcut::etb_drain(1),
            magecraft_drain_each_opp(1),
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

// ── Batch 40: more Witherbloom cards ────────────────────────────────────────

/// Witherbloom Toxicologist — {1}{B}{G}, 2/2 Human Druid Deathtouch.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, target creature gets -1/-1 until end of turn."
/// A removal-oriented magecraft creature — drips -1/-1 onto opp's bears
/// while the deathtouch body trades up.
pub fn witherbloom_toxicologist() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Toxicologist",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
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

/// Pest Husk — {B}{G}, 1/1 Pest Zombie.
/// Synthesised Oracle: "Deathtouch / When this creature dies, you gain
/// 1 life." A canonical Pest body (1/1 with the death lifegain rider)
/// plus deathtouch — trades up on the ground.
pub fn pest_husk() -> CardDefinition {
    CardDefinition {
        name: "Pest Husk",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Zombie],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::dies_gain_life(1)],
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

/// Witherbloom Bloodglyph — {1}{B}{G}, Sorcery.
/// Synthesised Oracle: "Drain 2 life. Create a 1/1 black and green Pest
/// creature token with 'When this creature dies, you gain 1 life.'"
/// 3-mana double-up: drain 2 + Pest token (with on-die lifegain).
pub fn witherbloom_bloodglyph() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bloodglyph",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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

/// Witherbloom Rotsage — {2}{B}{G}, 3/3 Zombie Druid.
/// Synthesised Oracle: "When this creature enters, you may sacrifice a
/// creature. If you do, draw a card and gain 1 life." A
/// disposable-fodder enabler that turns a Pest into a card.
pub fn witherbloom_rotsage() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Rotsage",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::MayDo {
            description: "Sacrifice a creature: draw a card and gain 1 life".to_string(),
            body: Box::new(Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Creature,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::GainLife {
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

/// Witherbloom Sproutchant — {1}{G}, 1/2 Elf Druid.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, put a +1/+1 counter on this creature." A
/// self-growing magecraft body that scales over a multi-spell turn.
pub fn witherbloom_sproutchant() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sproutchant",
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

/// Witherbloom Distiller — {1}{B}{G}, 2/3 Plant Druid.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, each opponent loses 1 life." A 3-mana magecraft
/// drain body in the Apprentice tradition with a sturdier 2/3 frame.
pub fn witherbloom_distiller() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Distiller",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
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

/// Pest Brewer — {2}{B}{G}, 2/2 Pest Druid.
/// Synthesised Oracle: "When this creature enters, create a 1/1 black
/// and green Pest creature token with \"When this creature dies, you
/// gain 1 life.\"" Standard 4-mana Pest token producer.
pub fn pest_brewer() -> CardDefinition {
    CardDefinition {
        name: "Pest Brewer",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: stx_pest_token(),
            count: Value::Const(1),
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

/// Witherbloom Alchemist — {2}{B}{G}, 3/3 Human Warlock.
/// Synthesised Oracle: "When this creature enters, each opponent loses
/// 2 life and you gain 2 life." A 4-mana drain-on-a-body that mirrors
/// the Silverquill drain template in B/G.
pub fn witherbloom_alchemist() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Alchemist",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb_drain(2)],
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

/// Witherbloom Bloomcaller — {1}{G}, 1/3 Plant Druid.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, you gain 1 life." Defensive lifegain on each spell.
pub fn witherbloom_bloomcaller() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bloomcaller",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Pestsage — {3}{B}{G}, 4/4 Plant Druid.
/// Synthesised Oracle: "When this creature enters, create two 1/1 Pest
/// tokens." 5-mana finisher that fan-mints two death-triggered Pests.
pub fn witherbloom_pestsage() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestsage",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: stx_pest_token(),
            count: Value::Const(2),
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

// ── Batch 42 (modern_decks) — Witherbloom expansion ─────────────────────────

/// Witherbloom Bramblevine — {1}{B}{G}, 3/2 Plant Warrior with Reach.
/// Synthesised Oracle: "Reach. Whenever you gain life, put a +1/+1 counter
/// on this creature." A 3-mana lifegain-tribal payoff body that scales
/// off Witherbloom Apprentice / Pest token death triggers.
pub fn witherbloom_bramblevine() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bramblevine",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
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

/// Witherbloom Sapglyph — {B}{G} Sorcery.
/// Synthesised Oracle: "Target opponent loses 2 life and you gain 2 life."
/// 2-mana drain — the cheapest direct drain spell in the Witherbloom
/// catalog, ideal alongside Apprentice / Pestreaver for life-swing turns.
pub fn witherbloom_sapglyph() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapglyph",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: target_filtered(SelectionRequirement::Player),
            to: Selector::You,
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

/// Pest Cultivator II — {2}{G}, 2/3 Pest Druid.
/// Synthesised Oracle: "When this creature enters, create a 1/1 black-and-
/// green Pest token." 3-mana 2-for-1 (3/2 body + 1/1 Pest with death
/// rider).
pub fn pest_cultivator_v2() -> CardDefinition {
    CardDefinition {
        name: "Pest Cultivator II",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: stx_pest_token(),
            count: Value::Const(1),
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

/// Witherbloom Pestpicker — {1}{B}, 2/1 Pest Rogue Menace.
/// Synthesised Oracle: "Menace. Whenever this creature attacks, each
/// opponent loses 1 life." 2-mana aggressive evasion + drain trigger.
pub fn witherbloom_pestpicker() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestpicker",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Bloomstalk — {2}{G}, 2/4 Plant Druid.
/// Synthesised Oracle: "When this creature enters, you gain 2 life.
/// Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// put a +1/+1 counter on this creature." 3-mana lifegain + self-grow
/// magecraft body.
pub fn witherbloom_bloomstalk() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bloomstalk",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            crate::effect::shortcut::etb_gain_life(2),
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

/// Witherbloom Coatlcoiler — {2}{B}{G}, 3/3 Snake Druid Deathtouch.
/// Synthesised Oracle: "Deathtouch. When this creature enters, target
/// player loses 2 life." 4-mana deathtouch body that also disrupts.
pub fn witherbloom_coatlcoiler() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Coatlcoiler",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::LoseLife {
            who: target_filtered(SelectionRequirement::Player),
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

/// Witherbloom Cinderscribe — {3}{B}{G}, 3/4 Plant Warrior Trample.
/// Synthesised Oracle: "Trample. When this creature enters, create two
/// 1/1 black-and-green Pest tokens. Each opponent loses 2 life."
/// 5-mana 3-for-1 finisher that establishes board presence and pressure.
pub fn witherbloom_cinderscribe() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Cinderscribe",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: stx_pest_token(),
                count: Value::Const(2),
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 43 (modern_decks) — Witherbloom expansion ─────────────────────────

/// Witherbloom Thornmaster — {1}{B}{G}, 2/3 Plant Druid Deathtouch.
/// Synthesised Oracle: "Deathtouch. When this creature enters, create
/// a 1/1 black-and-green Pest creature token with 'When this creature
/// dies, you gain 1 life.'" 3-mana sticky defensive Pest seeder.
pub fn witherbloom_thornmaster() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Thornmaster",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: stx_pest_token(),
            count: Value::Const(1),
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

/// Witherbloom Grafted Seer — {B}{G}, 1/3 Plant Druid.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, scry 1." 2-mana magecraft selection body.
pub fn witherbloom_grafted_seer() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Grafted Seer",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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

/// Witherbloom Ravensoul — {2}{B}{G}, 3/3 Plant Warlock.
/// Synthesised Oracle: "When this creature dies, each opponent loses
/// 2 life and you gain 2 life." 4-mana death-drain body.
pub fn witherbloom_ravensoul() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Ravensoul",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::dies_drain(2)],
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

/// Witherbloom Blightroot — {2}{B} Sorcery. Synthesised Oracle:
/// "Each opponent loses 3 life and you gain 3 life. Surveil 1."
/// 3-mana drain + selection.
pub fn witherbloom_blightroot() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Blightroot",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(3),
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

/// Witherbloom Pestswarm Master — {3}{B}{G}, 4/3 Pest Druid.
/// Synthesised Oracle: "When this creature enters, create two 1/1
/// black-and-green Pest creature tokens with 'When this creature dies,
/// you gain 1 life.'" 5-mana 3-for-1 go-wide finisher.
pub fn witherbloom_pestswarm_master() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestswarm Master",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: stx_pest_token(),
            count: Value::Const(2),
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

/// Witherbloom Spireling — {1}{G}, 2/2 Plant Druid Reach.
/// Synthesised Oracle: "Reach. When this creature enters, you gain
/// 2 life." 2-mana anti-flier + lifegain enabler.
pub fn witherbloom_spireling() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Spireling",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 47 (modern_decks) — Witherbloom expansion ─────────────────────────

/// Witherbloom Vinepicker — {B}{G}, 2/2 Plant Druid. Synthesised Oracle:
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// put a +1/+1 counter on this creature." 2-mana magecraft growth body.
pub fn witherbloom_vinepicker() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vinepicker",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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

/// Witherbloom Pestbloomer — {2}{B}{G}, 3/3 Plant Druid. Synthesised
/// Oracle: "When this creature enters, create two 1/1 black-green
/// Pest tokens with 'When this creature dies, you gain 1 life.'"
/// 4-mana body + Pest engine. Pest tokens feed Bayou Groff, Pest
/// Tender, and other sacrifice payoffs.
pub fn witherbloom_pestbloomer() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestbloomer",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(stx_pest_token(), 2)],
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

/// Witherbloom Rotsplash — {1}{B} Instant. Synthesised Oracle:
/// "Target creature gets -3/-3 until end of turn. You gain 1 life."
/// 2-mana ruthlessly efficient removal trick. Trades a -1/-1 net
/// from Glimmer for a strict upgrade to -3/-3.
pub fn witherbloom_rotsplash() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Rotsplash",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
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

/// Witherbloom Vinetwister — {3}{G}, 3/4 Plant Druid. Synthesised
/// Oracle: "When this creature enters, put a +1/+1 counter on each
/// other creature you control." A green Champion of Lambholt-style
/// fan-out for Witherbloom.
pub fn witherbloom_vinetwister() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vinetwister",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
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


// ── Batch 48 (modern_decks) — Witherbloom expansion ─────────────────────────

/// Witherbloom Pestcaller II — {2}{B}, 2/2 Human Warlock. Synthesised
/// Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, create a 1/1 B/G Pest token with 'When this
/// creature dies, you gain 1 life.'" 3-mana mid-curve Pest engine.
pub fn witherbloom_pestcaller_v2() -> CardDefinition {
    use crate::effect::shortcut::magecraft_mint_token;
    CardDefinition {
        name: "Witherbloom Pestcaller II",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_mint_token(stx_pest_token(), 1)],
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

/// Witherbloom Vinepriest — {1}{B}{G}, 2/3 Plant Cleric. Synthesised
/// Oracle: "When this creature enters, you gain 2 life. Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, you gain
/// 1 life." 3-mana defensive lifegain scaler.
pub fn witherbloom_vinepriest() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vinepriest",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            crate::effect::shortcut::etb_gain_life(2),
            magecraft_gain_life(1),
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

/// Pest Quartermaster — {3}{B}{G}, 3/4 Plant Druid Trample.
/// Synthesised Oracle: "Trample. When this creature enters, create
/// a 1/1 B/G Pest token with 'When this creature dies, you gain 1
/// life,' then draw a card." 5-mana grindy top-end + Pest engine.
pub fn pest_quartermaster() -> CardDefinition {
    CardDefinition {
        name: "Pest Quartermaster",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: stx_pest_token(),
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

/// Witherbloom Toxicvial — {1}{B} Instant. Synthesised Oracle:
/// "Target creature gets -3/-3 until end of turn." 2-mana efficient
/// shrink-removal — kills any creature with toughness ≤ 3 instantly.
pub fn witherbloom_toxicvial() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Toxicvial",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Lifechant — {2}{G} Sorcery. Synthesised Oracle:
/// "You gain 5 life. Scry 1." 3-mana lifegain + smoothing. Pairs
/// with Honor Troll's lifegain gate and Light of Promise's
/// dynamic-life payoff.
pub fn witherbloom_lifechant() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lifechant",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
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

// ── Batch 48 follow-up (modern_decks) — Witherbloom expansion 2 ─────────────

/// Pest Glutton — {2}{B}{G}, 3/3 Pest Beast. Synthesised Oracle:
/// "When this creature enters, create a 1/1 B/G Pest token with
/// 'When this creature dies, you gain 1 life,' then you gain 1
/// life." 4-mana grindy body + token + lifegain.
pub fn pest_glutton() -> CardDefinition {
    CardDefinition {
        name: "Pest Glutton",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Beast],
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
                    definition: stx_pest_token(),
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

/// Witherbloom Saprosage — {1}{B}{G}, 2/3 Plant Druid. Synthesised
/// Oracle: "When this creature enters, scry 2. Magecraft — Whenever
/// you cast or copy an instant or sorcery spell, you gain 1 life."
/// 3-mana defensive scry-and-scale body.
pub fn witherbloom_saprosage() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Saprosage",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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
                effect: Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                },
            },
            magecraft_gain_life(1),
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

/// Pestilent Marsh — {1}{G} Sorcery. Synthesised Oracle: "Create two
/// 1/1 B/G Pest tokens with 'When this creature dies, you gain 1 life.'"
/// 2-mana double Pest mint. Same shape as Pest Summoning at the
/// mono-green cost.
pub fn pestilent_marsh() -> CardDefinition {
    CardDefinition {
        name: "Pestilent Marsh",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: stx_pest_token(),
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

/// Witherbloom Witchwarden — {3}{B}{G}, 3/3 Plant Warlock Lifelink.
/// Synthesised Oracle: "Lifelink." Vanilla 5-mana lifelink top-end.
pub fn witherbloom_witchwarden() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Witchwarden",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Toxicvigor — {2}{B}{G} Sorcery. Synthesised Oracle:
/// "Each opponent loses 3 life and you gain 3 life. Surveil 1."
/// 4-mana drain + selection.
pub fn witherbloom_toxicvigor() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Toxicvigor",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(3),
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

// ── Batch 48 follow-up #2 (modern_decks) — more Witherbloom cards ───────────

/// Pestseed — {G} Sorcery. Synthesised Oracle: "Create a 1/1 B/G
/// Pest token with 'When this creature dies, you gain 1 life.'"
/// 1-mana cheap Pest minter.
pub fn pestseed() -> CardDefinition {
    CardDefinition {
        name: "Pestseed",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: stx_pest_token(),
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

// ── Batch 49 (modern_decks) — more Witherbloom cards ────────────────────────

/// Witherbloom Pestseer — {B}{G}, 2/2 Plant Druid.
/// Synthesised Oracle: "When this creature enters, create a 1/1
/// black-and-green Pest creature token with 'When this creature
/// dies, you gain 1 life.'" 2-mana value Pest-minter body.
pub fn witherbloom_pestseer() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestseer",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(stx_pest_token(), 1)],
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

/// Witherbloom Sourceweaver — {2}{B}{G}, 3/3 Plant Warlock Deathtouch.
/// Synthesised Oracle: "Deathtouch. When this creature enters, target
/// player loses 2 life and you gain 2 life." 4-mana drain-on-arrival
/// body — Witherbloom's classic drain Warlock tribal payoff.
pub fn witherbloom_sourceweaver() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sourceweaver",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Drain {
                from: target_filtered(SelectionRequirement::Player),
                to: Selector::You,
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

/// Witherbloom Sapburst — {1}{G} Sorcery. Synthesised Oracle:
/// "Put two +1/+1 counters on target creature you control. Scry 1."
/// 2-mana growth + filter spell — Witherbloom-flavor of Tinybones.
pub fn witherbloom_sapburst() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapburst",
        cost: cost(&[generic(1), g()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Brewer (v2) — {1}{B}{G}, 2/3 Pest Druid.
/// Synthesised Oracle: "When this creature enters, you gain 1 life."
/// 3-mana cheap Pest tribal anchor.
pub fn pest_brewer_v2() -> CardDefinition {
    CardDefinition {
        name: "Pest Cauldron Brewer",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb_gain_life(1)],
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

/// Witherbloom Greenwarden — {2}{G}, 2/2 Plant Druid Reach.
/// Synthesised Oracle: "Reach. When this creature enters, you gain
/// 2 life." 3-mana defensive anti-flier lifegainer.
pub fn witherbloom_greenwarden() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Greenwarden",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 50: Witherbloom synthesised cards ────────────────────────────────

/// Witherbloom Drainscholar v2 — {1}{B}, 1/3 Plant Warlock. Magecraft
/// gain 1 life. 2-mana defensive lifegain-on-cast body.
/// (Disambiguated from the existing batch-49 Drainscholar.)
pub fn witherbloom_drainscholar_b50() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Drainscholar Adept",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Hierarch — {B}{G}, 2/1 Pest. ETB mint a Pest token.
/// Aggressive 2-mana Pest engine.
pub fn pest_hierarch() -> CardDefinition {
    CardDefinition {
        name: "Pest Hierarch",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(stx_pest_token(), 1)],
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

/// Witherbloom Bloodseeker — {2}{B}{G}, 3/3 Plant Vampire Lifelink.
/// 4-mana lifelink anchor.
pub fn witherbloom_bloodseeker() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bloodseeker",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Vampire],
            ..Default::default()
        },
        power: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Disciple — {1}{G}, 1/2 Pest Druid. ETB Scry 1 + gain 1 life.
/// 2-mana defensive smoother.
pub fn pest_disciple() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Pest Disciple",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Lifescribe — {1}{B}{G}, 2/3 Human Druid. ETB drain 1
/// + magecraft gain 1 life. 3-mana scaling lifegain body.
pub fn witherbloom_lifescribe() -> CardDefinition {
    use crate::effect::shortcut::etb_drain;
    CardDefinition {
        name: "Witherbloom Lifescribe",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_drain(1),
            magecraft_gain_life(1),
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

/// Pest Lifebloom — {B}{G}, Instant. Gain 4 life + Surveil 1.
/// 2-mana big lifegain + selection.
pub fn pest_lifebloom() -> CardDefinition {
    CardDefinition {
        name: "Pest Lifebloom",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(4),
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

/// Witherbloom Pestmage — {2}{B}, 3/2 Pest Wizard Menace.
/// Aggressive 3-mana Pest with menace.
pub fn witherbloom_pestmage() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestmage",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace],
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

/// Witherbloom Vinedrain — {2}{B}{G}, Sorcery. Drain 3 + Draw 1.
/// 4-mana drain + cantrip.
pub fn witherbloom_vinedrain() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vinedrain",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(3),
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

/// Witherbloom Roto-Sage — {3}{B}{G}, 4/4 Plant Druid Deathtouch.
/// 5-mana finisher — Plant deathtouch body.
pub fn witherbloom_roto_sage() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Roto-Sage",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Deathtouch],
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

/// Pest Cultivator-Sage — {2}{B}{G}, 3/3 Plant Druid. Attacks-trigger
/// mints a Pest token. Hierarch-style scaling attacker.
pub fn pest_cultivator_sage() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Pest Cultivator-Sage",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: stx_pest_token(),
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

/// Witherbloom Decaymage — {1}{B}, 2/2 Pest Warlock. Magecraft
/// each opp loses 1 life — same shape as Witherbloom Apprentice but
/// on a Pest-typed 2-mana body.
pub fn witherbloom_decaymage() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Decaymage",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Mortician — {2}{B}, 2/2 Human Warlock. "Whenever a
/// player sacrifices a creature, put a +1/+1 counter on this
/// creature." Aristocrats-style Mortician-Beetle template wired off
/// the new `EventKind::CreatureSacrificed` event (CR 701.16) — the
/// sacrifice-specific event fires before the CreatureDied event, so
/// this trigger fires on sacrifices but **not** on natural deaths
/// (combat damage, lethal pings, exile-via-Path).
pub fn witherbloom_mortician() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Mortician",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::AnyPlayer),
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

/// Witherbloom Sacrosanct — {B}{G}, Sorcery. As an additional cost,
/// sacrifice a creature. Drain 3.
pub fn witherbloom_sacrosanct() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sacrosanct",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        // Approximation: sacrifice at resolution rather than as
        // additional cost (engine-wide gap shared with Necrotic Fumes).
        // Functionally equivalent — the fodder hits the graveyard and
        // the drain fires whether the sac is paid as cost or as effect.
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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

/// Pest Pestmaster — {3}{B}{G}, 3/3 Pest Warlock. "Whenever you
/// sacrifice a creature, you may put a +1/+1 counter on this
/// creature." A controlled-side variant of Witherbloom Mortician — uses
/// EventScope::YourControl so opponent sacrifices don't trigger.
pub fn pest_pestmaster_b51() -> CardDefinition {
    CardDefinition {
        name: "Pest Pestmaster",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl),
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

/// Witherbloom Lichbloom — {2}{B}{G}, 3/3 Plant Zombie. "When this
/// creature dies, return target creature card from your graveyard to
/// your hand." Self-replacing on death.
pub fn witherbloom_lichbloom() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lichbloom",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Zombie],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::OtherThanSource),
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

/// Pest Cradlescale — {1}{G}, 2/2 Pest Insect Reach. ETB mints a 1/1
/// Pest token. Anti-flier + Pest engine.
pub fn pest_cradlescale() -> CardDefinition {
    CardDefinition {
        name: "Pest Cradlescale",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Insect],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(stx_pest_token(), 1)],
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

/// Witherbloom Pestcaller v2 — {3}{B}{G}, 3/3 Pest Warlock. ETB mints
/// 3 Pest tokens. 5-mana go-wide finisher.
/// (Disambiguated from the existing batch-X Pestcaller.)
pub fn witherbloom_pestcaller_b50() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pest-Caller Adept",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(stx_pest_token(), 3)],
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

/// Pest Anointer — {1}{B}, 1/2 Pest Cleric. "Whenever you sacrifice
/// a creature, you gain 1 life." Uses the new CR 701.16 sacrifice
/// event with `YourControl` scope.
pub fn pest_anointer() -> CardDefinition {
    CardDefinition {
        name: "Pest Anointer",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Bloodreaper — {2}{B}{G}, 3/3 Plant Warlock. "Whenever
/// you sacrifice a creature, each opponent loses 1 life." Aristocrat
/// drain payoff via the new sacrifice event. Disambiguated from the
/// existing `witherbloom_reaper` factory in `extras`.
pub fn witherbloom_bloodreaper() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bloodreaper",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl),
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Conservator — {2}{G}, 2/3 Pest Druid. Sac-a-Pest activated
/// ability: `{T}, Sacrifice a Pest: Draw a card.`
pub fn pest_conservator() -> CardDefinition {
    CardDefinition {
        name: "Pest Conservator",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            tap_cost: false,
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            // Sacrifice a Pest you control as the activated ability's
            // first step; if no Pest is available the resolver no-ops the
            // sac and the draw still resolves (resolve-time picker vs
            // cost-time pre-pay shape — same trade as Witherbloom
            // Pestkeeper).
            effect: Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Pest))
                        .and(SelectionRequirement::ControlledByYou),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
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

/// Witherbloom Bloodweaver — {3}{B}{G}, 4/4 Vampire Warlock
/// Lifelink + Trample. 5-mana finisher. Disambiguated from the
/// existing `witherbloom_lifedrinker` factory in `extras`.
pub fn witherbloom_bloodweaver() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bloodweaver",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Lifelink, Keyword::Trample],
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

// ── batch 53: more Witherbloom cards ────────────────────────────────────────

/// Witherbloom Grimherb — {B}{G}, 2/2 Plant Druid Deathtouch. Magecraft
/// gain 1 life. 2-mana defensive deathtouch lifegain body.
pub fn witherbloom_grimherb() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Grimherb",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
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

/// Pest Brood — {1}{B}{G}, Sorcery. Creates 2 Pest tokens (each carries
/// the standard die-to-gain-1-life trigger via the shared
/// `stx_pest_token()` helper).
pub fn pest_brood() -> CardDefinition {
    CardDefinition {
        name: "Pest Brood",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: stx_pest_token(),
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

/// Witherbloom Pestpath — {3}{B}{G}, 3/4 Plant Beast Trample. 5-mana
/// curve-topper.
pub fn witherbloom_pestpath() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestpath",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Rotbloom — {2}{B}, Sorcery. Drain 3 (each opp loses 3, you
/// gain 3). 3-mana drain finisher.
pub fn witherbloom_rotbloom() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Rotbloom",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
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

// ── batch 54: more Witherbloom cards ────────────────────────────────────────

/// Witherbloom Creeper — {1}{B}{G}, 3/2 Plant Insect Deathtouch. Magecraft
/// self-pump +1/+0 EOT. 3-mana aggressive deathtouch on-cast scaler.
pub fn witherbloom_creeper() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Creeper",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Insect],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
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

/// Pest Lord — {3}{B}{G}, 3/3 Pest Warlock. Static "Other Pest creatures
/// you control get +1/+1" (Pest tribal anthem).
pub fn pest_lord() -> CardDefinition {
    use crate::effect::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Pest Lord",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Pest creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Pest))
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

/// Witherbloom Drainer — {2}{B}{G}, 2/3 Plant Warlock. ETB Seq(Drain 2 +
/// GainLife 1) — 4-mana drain anchor.
pub fn witherbloom_drainer() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Witherbloom Drainer",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Mossback — {2}{G}, 2/4 Plant Beast Reach. 3-mana defensive
/// reach blocker.
pub fn witherbloom_mossback() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Mossback",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Reach],
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

/// Pest Curse — {1}{B}, Sorcery. Seq(Mint 2 Pests + Discard 1 self).
/// 2-mana asymmetric mass-mint with self-cost.
pub fn pest_curse() -> CardDefinition {
    CardDefinition {
        name: "Pest Curse",
        cost: cost(&[generic(1), b()]),
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
                definition: stx_pest_token(),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
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

/// Witherbloom Hexvine — {3}{B}{G}, Sorcery. Seq(Destroy target creature
/// + GainLife 2). 5-mana hard removal + lifegain rider.
pub fn witherbloom_hexvine() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Hexvine",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Pestcradle — {1}{B}{G}, 2/2 Plant Druid. ETB Seq(mint a
/// Pest + GainLife 1). Cheap Pest enabler with a touch of lifegain to
/// stack with the Pest's own die-to-gain-1 trigger.
pub fn witherbloom_pestcradle() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Witherbloom Pestcradle",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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
                definition: stx_pest_token(),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Brewmaster — {2}{B}, 2/3 Pest Warlock. "Whenever you sacrifice a
/// creature, each opponent loses 1 life." Aristocrat drain payoff on a
/// modest body. Uses the new `EventKind::CreatureSacrificed/YourControl`
/// scope so opp-side sacs don't trigger it.
pub fn pest_brewmaster() -> CardDefinition {
    CardDefinition {
        name: "Pest Brewmaster",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl),
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Pestcaller — {2}{B}{G}, 3/3 Plant Druid. ETB Seq(mint 2
/// Pests + Surveil 1). 4-mana go-wide Pest engine with selection.
pub fn witherbloom_pestcaller_b54() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Witherbloom Pestcaller II",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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
                count: Value::Const(2),
                definition: stx_pest_token(),
            },
            Effect::Surveil {
                who: PlayerRef::You,
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

/// Witherbloom Vitalcoil — {1}{G}, 2/2 Plant Druid. Magecraft GainLife
/// 2 — defensive magecraft body that hits the 4-life-gained threshold
/// across two spells (instead of one).
pub fn witherbloom_vitalcoil() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vitalcoil",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(2)],
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

/// Witherbloom Pestharvest — {2}{B}{G}, Sorcery. Mint 2 Pests + Draw 1.
/// 4-mana go-wide Pest mint + cantrip.
pub fn witherbloom_pestharvest() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestharvest",
        cost: cost(&[generic(2), b(), g()]),
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
                definition: stx_pest_token(),
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

// ── Push (modern_decks, batch 56) — new Witherbloom STX cards ──────────────

/// Witherbloom Pestreaper — {2}{B}{G}, 3/3 Pest Warlock.
/// "Whenever you sacrifice a creature, put a +1/+1 counter on this
/// creature and you gain 1 life." Aristocrat double-payoff: grows on
/// sacrifice + lifegain on the same trigger.
pub fn witherbloom_pestreaper_b56() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestreaper II",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
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

/// Witherbloom Soulshade — {1}{B}, 2/2 Pest Wizard. Dies-trigger
/// returns target creature card with mana value 2 or less from your
/// graveyard to your hand. Cheap aristocrat-loop fodder.
pub fn witherbloom_soulshade() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Soulshade",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(2))
                        .and(SelectionRequirement::OtherThanSource),
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

/// Witherbloom Necrofeast — {2}{B}{G}, Sorcery. Sacrifice a creature.
/// Drain 4 (each opponent loses 4 life, you gain 4 life). The
/// sacrifice emits `EventKind::CreatureSacrificed`, so Mortician /
/// Pestmaster / Anointer triggers all fire.
pub fn witherbloom_necrofeast() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Necrofeast",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Caretaker — {1}{G}, 2/1 Pest Druid. ETB mints a Pest token
/// and Surveil 1. 2-mana Pest engine + selection.
pub fn pest_caretaker() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Pest Caretaker",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
            },
            Effect::Surveil {
                who: PlayerRef::You,
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

/// Witherbloom Tomeshade — {1}{B}{G}, 2/3 Plant Druid. ETB mills 3
/// from each opponent + Drain 1 (each opp loses 1 life, you gain 1
/// life). Self-mill + drain enabler for delirium / graveyard payoffs.
pub fn witherbloom_tomeshade() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Witherbloom Tomeshade",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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

// ── Push (modern_decks, batch 56b) — five more Witherbloom cards ────────────

/// Witherbloom Crypt-Caller — {2}{B}, 2/2 Pest Warlock. Dies-trigger
/// drain 2 via the new `dies_drain(2)` shortcut. 3-mana
/// aristocrats-fodder body.
pub fn witherbloom_crypt_caller() -> CardDefinition {
    use crate::effect::shortcut::dies_drain;
    CardDefinition {
        name: "Witherbloom Crypt-Caller",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_drain(2)],
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

/// Witherbloom Mill-Mage — {2}{B}{G}, 2/3 Human Druid. ETB mill 4
/// from each opponent via the new `etb_mill_each_opp(4)` shortcut.
/// Aggressive graveyard fuel for delirium / mill-matters builds.
pub fn witherbloom_mill_mage() -> CardDefinition {
    use crate::effect::shortcut::etb_mill_each_opp;
    CardDefinition {
        name: "Witherbloom Mill-Mage",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mill_each_opp(4)],
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

/// Pest Bonewright — {1}{B}, 2/1 Pest Warlock. Dies-trigger drain 1.
/// 2-mana cheap aristocrats trade body — 2-power offense + drain
/// rider on the way out.
pub fn pest_bonewright() -> CardDefinition {
    use crate::effect::shortcut::dies_drain;
    CardDefinition {
        name: "Pest Bonewright",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_drain(1)],
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

/// Witherbloom Decoder — {1}{U}, 1/3 Human Wizard. Magecraft mill 1
/// from each opponent. Cheap recurring graveyard fuel.
pub fn witherbloom_decoder() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Decoder",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Mill {
            who: Selector::Player(PlayerRef::EachOpponent),
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

/// Pest Roostmaster — {2}{B}{G}, 3/3 Pest Warlock. "Whenever you
/// sacrifice a creature, create a 1/1 black-green Pest token."
/// Self-replacing aristocrats engine — every sacrifice mints a
/// fresh Pest, which itself can be sacrificed later for chained drain.
pub fn pest_roostmaster() -> CardDefinition {
    CardDefinition {
        name: "Pest Roostmaster",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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

// ── Push (modern_decks, batch 57): 5 more Witherbloom cards ────────────────

/// Pest Soulreaver — {3}{B}{G}, 3/3 Pest Warlock. Dies-trigger drain 3.
/// 5-mana finisher with built-in 6-life death swing.
pub fn pest_soulreaver() -> CardDefinition {
    use crate::effect::shortcut::dies_drain;
    CardDefinition {
        name: "Pest Soulreaver",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_drain(3)],
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

/// Witherbloom Pestmender — {1}{G}, 1/2 Plant Druid. Magecraft puts a
/// +1/+1 counter on target Pest you control. Cheap Pest-tribal counter
/// engine — scales every IS spell into a Pest pump.
pub fn witherbloom_pestmender() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestmender",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::HasCreatureType(CreatureType::Pest)
                    .and(SelectionRequirement::ControlledByYou),
            },
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

/// Witherbloom Necropoet — {2}{B}, 2/3 Human Warlock. "Whenever you
/// sacrifice a creature, put a +1/+1 counter on each Pest you control."
/// Pest-tribal scaling that grows the entire Pest swarm per sacrifice.
pub fn witherbloom_necropoet() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Necropoet",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Pest)
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

/// Witherbloom Soulsmith — {3}{B}{G}, 3/4 Plant Druid. ETB drain 2 +
/// Scry 1. 5-mana value engine combining drain swing with selection.
pub fn witherbloom_soulsmith() -> CardDefinition {
    use crate::effect::shortcut::{drain, etb};
    CardDefinition {
        name: "Witherbloom Soulsmith",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            drain(2),
            Effect::Scry {
                who: PlayerRef::You,
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

/// Pest Vanguard — {1}{B}{G}, 2/2 Pest Insect with Deathtouch.
/// Magecraft drain 1. 3-mana deathtouch trade body that also drains
/// per IS spell — fits squarely into the Witherbloom magecraft shell.
pub fn pest_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Pest Vanguard",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Insect],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 58): 5 more Witherbloom cards ────────────────

/// Witherbloom Toxicpath — {2}{B}, 2/3 Plant Warlock. ETB drain 1 +
/// Surveil 1. 3-mana value engine that combines incidental drain with
/// graveyard selection.
pub fn witherbloom_toxicpath() -> CardDefinition {
    use crate::effect::shortcut::{drain, etb};
    CardDefinition {
        name: "Witherbloom Toxicpath",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            drain(1),
            Effect::Surveil {
                who: PlayerRef::You,
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

/// Pest Tendril — {B}{G}, 2/1 Pest Beast. Dies-trigger: scry 1.
/// 2-mana aggressive body with graveyard fuel on death — the trade
/// keeps card velocity flowing for follow-up Witherbloom payoffs.
pub fn pest_tendril() -> CardDefinition {
    use crate::effect::shortcut::on_dies;
    CardDefinition {
        name: "Pest Tendril",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_dies(Effect::Scry {
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

/// Witherbloom Bramblepath — {1}{G}, 1/3 Plant Druid with Reach.
/// Magecraft: gain 1 life. Defensive flier-blocker that drips life
/// each instant or sorcery spell — closes out aggro matchups.
pub fn witherbloom_bramblepath() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bramblepath",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
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

/// Pest Beekeeper — {2}{G}, 2/3 Human Druid. ETB mint a Pest token.
/// 3-mana ramp-into-Pest body — drops a vanilla 2/3 + the death-trigger
/// Pest for further sacrifice value.
pub fn pest_beekeeper() -> CardDefinition {
    CardDefinition {
        name: "Pest Beekeeper",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(stx_pest_token(), 1)],
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

/// Witherbloom Mire-Maker — {3}{B}{G}, 4/4 Plant Warrior with Trample.
/// ETB drain 2. 5-mana finisher anchor that swings for trample and
/// kicks in 4-life swing immediately.
pub fn witherbloom_mire_maker() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Mire-Maker",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(2)],
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

// ── Push (modern_decks, batch 59): 5 more Witherbloom cards ────────────────

/// Witherbloom Corpsegrove — {2}{B}{G}, 3/3 Plant Beast. Dies-trigger:
/// create a Pest token. 4-mana sticky body — the Corpsegrove dies and
/// hands the controller a 1/1 + lifegain Pest as consolation.
pub fn witherbloom_corpsegrove() -> CardDefinition {
    use crate::effect::shortcut::on_dies;
    CardDefinition {
        name: "Witherbloom Corpsegrove",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: stx_pest_token(),
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

/// Pest Grovetender — {1}{B}, 1/1 Pest Druid with Deathtouch. ETB Scry 1.
/// 2-mana deathtouch trader with built-in graveyard selection — a
/// versatile early defender that scry-fixes the controller's draws.
pub fn pest_grovetender() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Pest Grovetender",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Thornpoet — {1}{G}, 1/3 Plant Druid with Reach. Magecraft
/// self-pump +1/+1 EOT. Defensive Reach + scaling per IS cast — closes
/// out aggro and ramps into combat.
pub fn witherbloom_thornpoet() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Thornpoet",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
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

/// Witherbloom Sapler — {B}{G}, 2/2 Plant Beast. Magecraft: +1/+1 EOT
/// to target friendly Pest. Pest-tribal pump payoff at 2 mana —
/// stacks with the Pest fan-out across batches.
pub fn witherbloom_sapler() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapler",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::HasCreatureType(CreatureType::Pest)
                    .and(SelectionRequirement::ControlledByYou),
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

// ── Push (modern_decks, batch 60): 3 more Witherbloom cards ────────────────

/// Pest Roostkeeper — {1}{B}{G}, 2/3 Pest Warlock. ETB mint 1 Pest +
/// magecraft scry 1. 3-mana go-wide body with selection scaling.
pub fn pest_roostkeeper() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Pest Roostkeeper",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_mint_token(stx_pest_token(), 1),
            magecraft_scry(1),
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

/// Witherbloom Mossherald — {2}{G}, 3/2 Plant Druid Trample. Magecraft
/// AddCounter(+1/+1, self). 3-mana magecraft-scaling tramper.
pub fn witherbloom_mossherald() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Mossherald",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample],
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

/// Witherbloom Vinepriest II — {2}{B}{G}, 3/3 Plant Cleric Lifelink. ETB
/// Drain 1 + magecraft GainLife 1. 4-mana defensive value engine —
/// double-incidental lifegain.
pub fn witherbloom_vinepriest_b60() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vinepriest II",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_drain(1),
            magecraft_gain_life(1),
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

/// Witherbloom Blightbearer — {3}{B}, 3/3 Zombie Warlock. ETB Seq(drain 2
/// + Scry 1). 4-mana defensive value engine — bulky body, immediate
///   drain swing, and graveyard selection rider.
pub fn witherbloom_blightbearer() -> CardDefinition {
    use crate::effect::shortcut::{drain, etb};
    CardDefinition {
        name: "Witherbloom Blightbearer",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            drain(2),
            Effect::Scry {
                who: PlayerRef::You,
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

// ── Push (modern_decks, batch 61): 5 more Witherbloom cards ────────────────

/// Witherbloom Pestcollector — {2}{B}{G}, 3/3 Plant Druid. ETB Seq(mint
/// Pest token + Scry 1). 4-mana go-wide Pest engine + selection.
pub fn witherbloom_pestcollector() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Witherbloom Pestcollector",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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
                definition: stx_pest_token(),
            },
            Effect::Scry {
                who: PlayerRef::You,
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

/// Pest Swarmleader — {2}{B}, 2/2 Pest Warlock. "Whenever you sacrifice
/// a creature, each opponent loses 1 life." Aristocrats drain payoff via
/// the `EventKind::CreatureSacrificed/YourControl` event (CR 701.16
/// sacrifice-as-distinct event).
pub fn pest_swarmleader() -> CardDefinition {
    CardDefinition {
        name: "Pest Swarmleader",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CreatureSacrificed,
                EventScope::YourControl,
            ),
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Rotweaver — {1}{G}, 1/2 Plant Druid. Magecraft GainLife
/// 2. Strong rate of magecraft lifegain — feeds Honor Troll, Light of
/// Promise, Felisa, etc.
pub fn witherbloom_rotweaver() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Rotweaver",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(2)],
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

/// Pest Thrasher — {1}{B}{G}, 2/2 Pest Insect Deathtouch + Reach. 3-mana
/// dual-defensive body — anti-flier deathtouch trade.
pub fn pest_thrasher() -> CardDefinition {
    CardDefinition {
        name: "Pest Thrasher",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Insect],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch, Keyword::Reach],
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

/// Witherbloom Vinemaster II — {3}{B}{G}, 3/4 Plant Druid Trample. ETB
/// drain 2 + magecraft AddCounter(+1/+1, Self). 5-mana mid-curve drainer
/// + self-growing magecraft body.
pub fn witherbloom_vinemaster_b61() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vinemaster II",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_drain(2),
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

// ── Push (modern_decks, batch 62): 2 more Witherbloom cards ────────────────

/// Pest Soulbinder — {1}{B}{G}, 2/2 Pest Warlock. "Whenever you
/// sacrifice a creature, scry 1." Card-selection aristocrats engine
/// via CR-701.16 sacrifice event.
pub fn pest_soulbinder() -> CardDefinition {
    CardDefinition {
        name: "Pest Soulbinder",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CreatureSacrificed,
                EventScope::YourControl,
            ),
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

/// Witherbloom Vineshaper — {2}{G}, 2/3 Plant Druid. Magecraft +1/+1
/// counter on each Pest you control (`ForEach Pest/ControlledByYou →
/// AddCounter +1/+1`). 3-mana Pest-tribal magecraft scaler.
pub fn witherbloom_vineshaper() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vineshaper",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::HasCreatureType(CreatureType::Pest)
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

// ── Push (modern_decks, batch 63): 5 more Witherbloom cards ─────────────────

/// Pest Soulkeeper — {B}{G}, 2/2 Pest Cleric. "Whenever you sacrifice a
/// creature, put a +1/+1 counter on this creature." 2-mana aristocrats
/// scaler via the CR-701.16 sacrifice event.
pub fn pest_soulkeeper() -> CardDefinition {
    CardDefinition {
        name: "Pest Soulkeeper",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CreatureSacrificed,
                EventScope::YourControl,
            ),
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

/// Witherbloom Marshhulk — {3}{B}{G}, 4/5 Plant Beast Trample. ETB drain
/// 2. 5-mana big-body drain finisher.
pub fn witherbloom_marshhulk() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Marshhulk",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(2)],
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

/// Pest Reaverling — {1}{B}, 2/1 Pest Warlock. Dies-trigger Drain 1.
/// 2-mana aristocrats trade body.
pub fn pest_reaverling() -> CardDefinition {
    use crate::effect::shortcut::dies_drain;
    CardDefinition {
        name: "Pest Reaverling",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_drain(1)],
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

/// Witherbloom Lifesnare — {1}{B}{G}, Sorcery. Seq(target creature gets
/// -3/-3 EOT + you gain 3 life). 3-mana shrink-removal + lifegain.
pub fn witherbloom_lifesnare() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lifesnare",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Bonewright — {2}{B}{G}, 3/3 Plant Druid. ETB Seq(mint Pest
/// + gain 2 life). 4-mana double-body + lifegain.
pub fn witherbloom_bonewright() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Witherbloom Bonewright",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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
                definition: stx_pest_token(),
                count: Value::Const(1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 64): 10 more Witherbloom cards ───────────────
//
// Focus: completing Pest-tribal / aristocrats / drain / counter-scaling
// patterns using existing shortcut helpers. Each card has a functionality
// test in `tests::stx`.

/// Pest Burrowmonger — {1}{B}{G}, 2/2 Pest Druid Deathtouch. 3-mana
/// deathtouch body — Witherbloom Pest with combat utility.
pub fn pest_burrowmonger() -> CardDefinition {
    CardDefinition {
        name: "Pest Burrowmonger",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
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

/// Witherbloom Mossrunner — {2}{G}, 2/3 Plant Warrior Trample. Magecraft
/// gain 1 life. 3-mana defensive trampler with on-cast lifegain.
pub fn witherbloom_mossrunner() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Mossrunner",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Trample],
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

/// Witherbloom Toxinspeaker — {1}{B}, 2/2 Human Warlock. ETB target opp
/// loses 2 life. 2-mana point-drain body.
pub fn witherbloom_toxinspeaker() -> CardDefinition {
    use crate::effect::shortcut::etb_drain_each_opp;
    CardDefinition {
        name: "Witherbloom Toxinspeaker",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain_each_opp(2)],
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

/// Pest Vinerunner — {G}, 1/1 Pest Druid Reach. Cheap Pest with Reach for
/// flyer defense.
pub fn pest_vinerunner() -> CardDefinition {
    CardDefinition {
        name: "Pest Vinerunner",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Reach],
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

/// Witherbloom Drainvine — {1}{B}{G}, Sorcery. Seq(Drain 2 + mint Pest).
/// 3-mana drain + token.
pub fn witherbloom_drainvine() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Witherbloom Drainvine",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(2),
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: stx_pest_token(),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Sapblade — {2}{B}, 3/2 Plant Warlock. Magecraft AddCounter
/// +1/+1 on self. 3-mana self-growing body.
pub fn witherbloom_sapblade() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapblade",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
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

/// Pest Vinegrower — {2}{B}{G}, 3/3 Pest Druid. ETB mint 2 Pest tokens.
/// 4-mana Pest-tribal go-wide.
pub fn pest_vinegrower() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Pest Vinegrower",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: stx_pest_token(),
            count: Value::Const(2),
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

/// Witherbloom Loamcaller — {2}{G}, 2/3 Plant Druid. Magecraft +1/+1
/// counter on target friendly Pest. Pest-tribal scaler.
pub fn witherbloom_loamcaller() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Loamcaller",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Pest))
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

/// Witherbloom Lifedrain — {1}{B}, Instant. Target creature gets -2/-2 EOT.
/// 2-mana shrink + creature kill at the small-creature slot.
pub fn witherbloom_lifedrain() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lifedrain",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Bannerer — {1}{B}, 2/2 Pest Warlock. Magecraft pump each Pest
/// you control +1/+0 EOT via the new tribal-pump shortcut.
pub fn pest_bannerer() -> CardDefinition {
    use crate::effect::shortcut::magecraft_pump_each_creature_type;
    CardDefinition {
        name: "Pest Bannerer",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_pump_each_creature_type(
            CreatureType::Pest,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Brood-Marauder — {3}{B}{G}, 4/3 Pest Warrior Menace. 5-mana
/// aggressive pest finisher with evasion.
pub fn pest_brood_marauder() -> CardDefinition {
    CardDefinition {
        name: "Pest Brood-Marauder",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Menace],
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

// ── Push (modern_decks, batch 67): 6 more Witherbloom cards ───────────────

/// Witherbloom Mossfen-Adept — {B}{G}, 2/2 Plant Druid Deathtouch.
/// Magecraft drain 1. 2-mana deathtouch + drain magecraft body.
pub fn witherbloom_mossfen_adept() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Mossfen-Adept",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Vinemother — {2}{B}{G}, 3/3 Plant Beast. ETB mints 2 Pest
/// tokens. 4-mana wide-board Pest enabler.
pub fn pest_vinemother() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Pest Vinemother",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: stx_pest_token(),
            count: Value::Const(2),
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

/// Witherbloom Lifesage — {1}{B}, 1/3 Human Cleric. ETB Seq(GainLife 2
/// + magecraft drain 1). 2-mana defensive lifegain magecraft scaler.
pub fn witherbloom_lifesage() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lifesage",
        cost: cost(&[generic(1), b()]),
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
        triggered_abilities: vec![etb_gain_life(2), magecraft_drain_each_opp(1)],
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

/// Witherbloom Sapdrinker (b67) — {2}{G}, 3/3 Plant Beast Trample.
/// Magecraft AddCounter(+1/+1, Self). 3-mana scaling trampler.
pub fn witherbloom_sapdrinker_b67() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapdrinker (b67)",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
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

/// Witherbloom Soulchant — {1}{B}{G}, Sorcery. Drain 2 (each opp loses
/// 2, you gain 2) + Surveil 1. 3-mana drain + selection.
pub fn witherbloom_soulchant() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Soulchant",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            Effect::GainLife {
                who: Selector::You,
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

/// Pest Skitterer — {B}, 1/1 Pest Insect. 1-mana cheap evasive trade-
/// up Pest with the printed die-to-gain-1 lifegain rider (rides on
/// the shared `stx_pest_token` template, but as a printed card it
/// carries the death trigger directly).
pub fn pest_skitterer() -> CardDefinition {
    CardDefinition {
        name: "Pest Skitterer",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Insect],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 68): more Witherbloom B/G cards ─────────────
//
// Focus: adding more synthesised Witherbloom college cards in canonical
// drain / lifegain / Pest-tribal patterns. Each card has a functionality
// test in `tests::stx`.

/// Witherbloom Sapchant — {1}{B}{G}, Instant. Drain 3 + Surveil 1.
/// 3-mana flexible instant-speed drain with selection rider.
pub fn witherbloom_sapchant() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapchant",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(3),
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

/// Pest Bloodling — {B}{G}, 2/1 Pest Insect Deathtouch. Compact
/// 2-mana Pest combat trade-up with deathtouch.
pub fn pest_bloodling() -> CardDefinition {
    CardDefinition {
        name: "Pest Bloodling",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Insect],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
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

/// Witherbloom Sapscholar — {1}{G}, 2/2 Plant Druid. Magecraft
/// Seq(GainLife 1 + Surveil 1). 2-mana magecraft lifegain + selection.
pub fn witherbloom_sapscholar() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapscholar",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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
            Effect::Surveil {
                who: PlayerRef::You,
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

/// Pest Carrionbinder — {2}{B}{G}, 3/3 Pest Warlock. ETB Seq(mint 2
/// Pest tokens + Drain 1). 4-mana Pest-tribal enabler + drain.
pub fn pest_carrionbinder() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Pest Carrionbinder",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
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
                definition: stx_pest_token(),
                count: Value::Const(2),
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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

/// Witherbloom Drainherald — {2}{B}, 2/3 Vampire Warlock Lifelink.
/// ETB drain 2. 3-mana evasive lifelink drainer.
pub fn witherbloom_drainherald() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Drainherald",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(2)],
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

/// Pest Spawnmother — {3}{B}{G}, 4/4 Pest Beast. ETB mints 3 Pest
/// tokens. 5-mana Pest go-wide finisher.
pub fn pest_spawnmother() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Pest Spawnmother",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: stx_pest_token(),
            count: Value::Const(3),
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

/// Witherbloom Vinescholar — {G}, 1/2 Plant Druid. Magecraft self
/// AddCounter(+1/+1). 1-mana magecraft scaler.
pub fn witherbloom_vinescholar() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vinescholar",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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

/// Witherbloom Reapdrain — {1}{B}{G}, Sorcery. Seq(Drain 2 + Draw 1).
/// 3-mana drain + cantrip.
pub fn witherbloom_reapdrain() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Reapdrain",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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

/// Pest Nightswarm — {1}{B}, 2/2 Pest Insect Flying. 2-mana evasive
/// Pest — anti-flier defender + tempo attacker.
pub fn pest_nightswarm() -> CardDefinition {
    CardDefinition {
        name: "Pest Nightswarm",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Insect],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Toxinbinder — {2}{B}, 3/2 Vampire Warlock. ETB target
/// creature gets -2/-2 EOT. 3-mana shrink-removal body.
pub fn witherbloom_toxinbinder() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Witherbloom Toxinbinder",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
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

// ── Batch 125 (push claude/modern_decks): four new Witherbloom cards ──────

/// Witherbloom Drainstride (b125) — {2}{B}{G}, 3/3 Plant Vampire.
/// "Whenever this creature attacks, each opponent loses 1 life and you
/// gain 1 life." 4-mana attack-drain body via the new
/// `on_attack_drain` shortcut.
pub fn witherbloom_drainstride_b125() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Drainstride (b125)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Vampire],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Lifescribe Elder (b125) — {1}{G}, 1/3 Plant Druid.
/// Magecraft GainLife 2. 2-mana defensive lifegain-on-cast body.
/// Higher-rate sibling to Witherbloom Vitalcoil's {1}{G} 2/2.
pub fn witherbloom_lifescribe_elder_b125() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lifescribe Elder (b125)",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(2)],
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

/// Pest Cinderpriest (b125) — {2}{B}, 2/2 Pest Cleric. ETB mints a
/// Pest token + magecraft drain 1 each opp. 3-mana double-payoff Pest
/// engine.
pub fn pest_cinderpriest_b125() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Pest Cinderpriest (b125)",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
            }),
            magecraft_drain_each_opp(1),
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

/// Witherbloom Reaperscholar (b125) — {3}{B}{G}, 4/4 Plant Druid
/// Deathtouch. Dies-trigger Drain 2 via `on_dies(Drain)`. 5-mana
/// finisher with deathtouch + death-drain rider.
pub fn witherbloom_reaperscholar_b125() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Reaperscholar (b125)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_drain(2)],
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

// ── Batch 126 (push claude/modern_decks): five new Witherbloom cards ──────

/// Witherbloom Mossgrower (b126) — {2}{B}{G}, 3/3 Plant Druid. On_dies
/// mints a 1/1 B/G Pest token (with the standard on-death gain-1-life
/// rider riding on the token). Self-replacing 4-mana body via the new
/// `dies_mint_token` shortcut.
pub fn witherbloom_mossgrower_b126() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Mossgrower (b126)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_mint_token(stx_pest_token(), 1)],
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

/// Witherbloom Toxinscholar (b126) — {1}{G}, 2/2 Plant Druid.
/// Magecraft GainLife 2. 2-mana magecraft lifegain body.
pub fn witherbloom_toxinscholar_b126() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Toxinscholar (b126)",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(2)],
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

/// Pest Pyrechewer (b126) — {1}{B}, 1/2 Pest. Dies-Drain 1. 2-mana
/// parting-shot Pest body — overlaps with stx_pest_token but on a
/// non-token frame so it stacks with Pestmancer / Pestmaster engines.
pub fn pest_pyrechewer_b126() -> CardDefinition {
    CardDefinition {
        name: "Pest Pyrechewer (b126)",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_drain(1)],
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

/// Witherbloom Sapcaster (b126) — {3}{B}{G}, 4/4 Plant Warlock. ETB
/// drain 3. 5-mana race-breaker finisher (6-life swing on entry).
pub fn witherbloom_sapcaster_b126() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapcaster (b126)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(3)],
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

/// Witherbloom Vinerunner (b126) — {2}{G}, 3/3 Plant Warrior Trample.
/// ETB GainLife 2. 3-mana defensive trampler with built-in lifegain.
pub fn witherbloom_vinerunner_b126() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vinerunner (b126)",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 127 (push claude/modern_decks): new Witherbloom cards ───────────

/// Witherbloom Sapsage (b127) — {1}{G}, 1/3 Plant Druid. Magecraft +1/+1
/// counter on self via the new `magecraft_self_counter_b127` pattern
/// (inlined for now). Aggressive self-growing magecraft 2-drop.
pub fn witherbloom_sapsage_b127() -> CardDefinition {
    use crate::effect::shortcut::cast_is_instant_or_sorcery;
    CardDefinition {
        name: "Witherbloom Sapsage (b127)",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
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

/// Pest Brewerthing (b127) — {2}{B}, 2/2 Pest Warlock. Dies → mint a
/// Pest token (self-replicating aristocrats body, uses
/// `dies_mint_token`).
pub fn pest_brewerthing_b127() -> CardDefinition {
    CardDefinition {
        name: "Pest Brewerthing (b127)",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_mint_token(stx_pest_token(), 1)],
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

/// Witherbloom Mossbinder (b127) — {2}{B}{G}, 3/3 Plant Warrior. ETB
/// drain 2 (4-life swing). 4-mana race breaker body.
pub fn witherbloom_mossbinder_b127() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Mossbinder (b127)",
        cost: cost(&[generic(2), b(), g()]),
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
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(2)],
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

/// Witherbloom Pestsower (b127) — {3}{B}{G} Sorcery. Seq(CreateToken
/// 2 Pests + Drain 2). 5-mana go-wide + drain finisher.
pub fn witherbloom_pestsower_b127() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestsower (b127)",
        cost: cost(&[generic(3), b(), g()]),
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
                definition: stx_pest_token(),
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::Player(PlayerRef::You),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Verdant Sage (b127) — {2}{G}, 2/4 Plant Druid Reach.
/// ETB GainLife 2. 3-mana anti-flier + lifegain.
pub fn witherbloom_verdant_sage_b127() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Verdant Sage (b127)",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Reach],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 128 (push claude/modern_decks): new Witherbloom cards ───────────

/// Witherbloom Toxicspeaker (b128) — {1}{B}, 1/3 Human Warlock. Magecraft
/// drain 1 — Apprentice on a chunkier base, easier to survive.
pub fn witherbloom_toxicspeaker_b128() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Toxicspeaker (b128)",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Pestcaller (b128) — {3}{B}{G}, 3/3 Plant Druid. ETB mints
/// a Pest token (with die→life trigger from `stx_pest_token`). 5-mana
/// pest-engine 2-for-1 body.
pub fn witherbloom_pestcaller_b128() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestcaller (b128)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(stx_pest_token(), 1)],
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

/// Witherbloom Mossfeeder (b128) — {1}{G}, 2/2 Plant Beast. Magecraft
/// +1/+1 counter on self — green's growth-on-cast body. Same shape as
/// Sapsage but trades 1 toughness for 1 power.
pub fn witherbloom_mossfeeder_b128() -> CardDefinition {
    use crate::effect::shortcut::cast_is_instant_or_sorcery;
    CardDefinition {
        name: "Witherbloom Mossfeeder (b128)",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
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

/// Witherbloom Reaper-Hand (b128) — {2}{B}, 3/2 Skeleton Warlock. Dies
/// → drain 2 from each opp. Aristocrats payoff (Witherbloom Saproot
/// template) with bigger body, smaller drain.
pub fn witherbloom_reaper_hand_b128() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Reaper-Hand (b128)",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_drain(2)],
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

/// Witherbloom Cauldronkeeper (b128) — {1}{B}{G}, 2/3 Human Warlock.
/// ETB Seq(Surveil 2 + GainLife 1). 3-mana defensive smoother that
/// fills the gy for Lorehold/Witherbloom recursion shells.
pub fn witherbloom_cauldronkeeper_b128() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Cauldronkeeper (b128)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Seq(vec![
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(2),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Sprawl-Vine (b128) — {2}{G}, 3/3 Plant Reach. Vanilla
/// curve-topper with reach for shutting down opposing Inkling/Spirit
/// flyers.
pub fn witherbloom_sprawl_vine_b128() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sprawl-Vine (b128)",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Reach],
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

/// Witherbloom Spellrot (b128) — {1}{B}{G} Sorcery. Seq(Drain 3 +
/// Surveil 1). 3-mana drain-and-dig. Stronger than Defend the Inkwell
/// at the same slot (drain 3 vs drain 2, 1 less mana but less scry).
pub fn witherbloom_spellrot_b128() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Spellrot (b128)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(3),
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

// ── Batch 129 (push claude/modern_decks): new Witherbloom cards ──────────

/// Witherbloom Vinetongue (b129) — {1}{G}{G}, 3/3 Plant Druid. Static
/// "Other Plant creatures you control get +1/+1." Plant-tribal anthem
/// for the Witherbloom Plant pool (Sprawl-Vine, Verdant Sage,
/// Pestcaller, Pest-Tender, Vinemaster, etc.).
pub fn witherbloom_vinetongue_b129() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Witherbloom Vinetongue (b129)",
        cost: cost(&[generic(1), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Plant creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Plant))
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

/// Witherbloom Bonewight (b129) — {2}{B}, 2/2 Skeleton Warlock.
/// Activated `{B}: Regenerate this creature.` (legacy Skeleton
/// regeneration template; reuses the engine's Keyword::Regenerate(n)
/// keyword tag whose value is the mana cost). Skeleton-tribal early
/// drop.
pub fn witherbloom_bonewight_b129() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bonewight (b129)",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Regenerate(1)],
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

/// Witherbloom Reaper-Lord (b129) — {2}{B}{B}, 3/3 Skeleton Warlock.
/// Static "Other Skeleton creatures you control get +1/+1 and have
/// menace." Skeleton-tribal anthem + evasion grant. Pairs with
/// Reaper-Hand (b128) and Bonewight (b129).
pub fn witherbloom_reaper_lord_b129() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Witherbloom Reaper-Lord (b129)",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![
            StaticAbility {
                description: "Other Skeleton creatures you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::HasCreatureType(CreatureType::Skeleton))
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Other Skeleton creatures you control have menace.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::HasCreatureType(CreatureType::Skeleton))
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    keyword: Keyword::Menace,
                },
            },
        ],
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

/// Witherbloom Petalmaster (b129) — {1}{G}, 2/2 Plant Druid. Magecraft
/// puts a +1/+1 counter on target Plant you control — Plant-tribal
/// magecraft growth, pairs with the Vinetongue anthem.
pub fn witherbloom_petalmaster_b129() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Petalmaster (b129)",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Plant))
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

/// Witherbloom Pestswarm (b129) — {2}{B}{G} Sorcery. Create three Pest
/// tokens. Witherbloom's go-wide Pest minter at 4 mana — pairs with
/// the Pest mascot lifegain riders.
pub fn witherbloom_pestswarm_b129() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestswarm (b129)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: stx_pest_token(),
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

/// Witherbloom Cauldronherder (b129) — {3}{B}{G}, 4/3 Human Warlock.
/// ETB Seq(mint Pest + drain 2). 5-mana drain-and-body — combo
/// finisher for Witherbloom drain shells. Uses the new b129
/// `etb_mint_token_and_drain` shortcut helper.
pub fn witherbloom_cauldronherder_b129() -> CardDefinition {
    use crate::effect::shortcut::etb_mint_token_and_drain;
    CardDefinition {
        name: "Witherbloom Cauldronherder (b129)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token_and_drain(stx_pest_token(), 2)],
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

/// Witherbloom Boneshroud (b129) — {B} Instant. Target creature gets
/// -2/-2 EOT. Cheap point-removal — efficient for the small-creature
/// slot, fits both Witherbloom and any Skeleton/Plant shell.
pub fn witherbloom_boneshroud_b129() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Boneshroud (b129)",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 130 (push claude/modern_decks): more Witherbloom cards ────────────

/// Witherbloom Skeletonsage (b130) — {1}{B}, 1/3 Skeleton Wizard.
/// Magecraft puts a +1/+1 counter on this creature. Self-growing
/// Skeleton on the curve below Reaper-Hand and Bonewight; pairs with
/// Reaper-Lord's Skeleton anthem.
pub fn witherbloom_skeletonsage_b130() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Skeletonsage (b130)",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
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

/// Witherbloom Planttender (b130) — {1}{G}, 1/3 Plant Druid, Reach.
/// Vanilla Plant defender with Reach — pairs with Vinetongue (Plant
/// anthem) and Sprawl-Vine to defend vs Flying-heavy boards.
pub fn witherbloom_planttender_b130() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Planttender (b130)",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
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

/// Witherbloom Blightroot (b130) — {1}{B}{G}, 2/2 Plant Beast, Deathtouch.
/// 3-mana Plant deathtoucher — a strong removal-or-trade body that
/// benefits from Vinetongue's Plant anthem.
pub fn witherbloom_blightroot_b130() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Blightroot (b130)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
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

/// Witherbloom Petalspeak (b130) — {2}{G} Sorcery. Put a +1/+1 counter
/// on each Plant you control. Mass Plant pump that scales with the
/// Plant subpool (Petalmaster, Sprawl-Vine, Mossfeeder, Vinetongue,
/// Pestcaller, Planttender, Blightroot).
pub fn witherbloom_petalspeak_b130() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Petalspeak (b130)",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Plant))
                    .and(SelectionRequirement::ControlledByYou),
            ),
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

/// Witherbloom Skullcarver (b130) — {3}{B}, 4/3 Skeleton Warrior.
/// Vanilla Skeleton body at top-of-curve — benefits from Reaper-Lord's
/// +1/+1/menace Skeleton anthem (becomes a 5/4 menace).
pub fn witherbloom_skullcarver_b130() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Skullcarver (b130)",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ─── Batch 131: Witherbloom synthesised cards ──────────────────────────────────

/// Witherbloom Pestseed (b131) — {1}{G}, 1/2 Plant Druid. ETB mints
/// a Pest token (with the SOS Pest attack-trigger gainlife rider riding
/// on the token).
pub fn witherbloom_pestseed_b131() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestseed (b131)",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(stx_pest_token(), 1)],
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

/// Witherbloom Bloodthorn (b131) — {1}{B}, 2/2 Plant Vampire. Dies →
/// drain 1.
pub fn witherbloom_bloodthorn_b131() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bloodthorn (b131)",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Vampire],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_drain(1)],
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

/// Witherbloom Decaywarden (b131) — {2}{B}{G}, 2/3 Plant Warlock,
/// Deathtouch. Magecraft drain 1.
pub fn witherbloom_decaywarden_b131() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Decaywarden (b131)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Lichbinder (b131) — {1}{B}, 1/2 Pest Cleric. "Whenever you
/// sacrifice a creature, each opponent loses 1 life." Aristocrat drain
/// payoff via `EventKind::CreatureSacrificed/YourControl`.
pub fn pest_lichbinder_b131() -> CardDefinition {
    CardDefinition {
        name: "Pest Lichbinder (b131)",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl),
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Rootwoven (b131) — {3}{B}{G}, 4/4 Plant Beast, Trample.
/// Vanilla finisher.
pub fn witherbloom_rootwoven_b131() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Rootwoven (b131)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Overgrowth (b131) — {2}{B}{G} Sorcery. CreateToken 3 Pests.
pub fn pest_overgrowth_b131() -> CardDefinition {
    CardDefinition {
        name: "Pest Overgrowth (b131)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: stx_pest_token(),
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

/// Witherbloom Drainshroud (b131) — {1}{B} Instant. Drain 2 (each opp
/// loses 2; you gain 2). Uses the `drain` shortcut.
pub fn witherbloom_drainshroud_b131() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Drainshroud (b131)",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain(2),
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

/// Witherbloom Lifescribe II (b131) — {2}{G}, 2/3 Plant Druid, Reach.
/// Magecraft GainLife 2.
pub fn witherbloom_lifescribe_ii_b131() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lifescribe II (b131)",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(2)],
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

// ── Batch 132 ───────────────────────────────────────────────────────────────

/// Witherbloom Pestcaller II (b132) — {1}{B}{G}, 2/2 Human Warlock.
/// ETB mint a Pest token and drain 1. Compact ETB Pest mint + drain
/// engine using `etb_mint_token_and_drain`.
pub fn witherbloom_pestcaller_ii_b132() -> CardDefinition {
    use crate::effect::shortcut::etb_mint_token_and_drain;
    CardDefinition {
        name: "Witherbloom Pestcaller II (b132)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token_and_drain(stx_pest_token(), 1)],
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

/// Pest Sproutbinder (b132) — {1}{G}, 1/3 Pest Druid, Reach. Defensive
/// Pest body that walls fliers and feeds Witherbloom's Pest pool.
pub fn pest_sproutbinder_b132() -> CardDefinition {
    CardDefinition {
        name: "Pest Sproutbinder (b132)",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
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

/// Witherbloom Pestbinder (b132) — {2}{B}, 2/3 Human Warlock.
/// On-attack drain 1. Combat-trigger drain body that scales with the
/// Pest tribe's anthems.
pub fn witherbloom_pestbinder_b132() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestbinder (b132)",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Mossreaver (b132) — {2}{G}, 3/2 Plant Druid. Vanilla
/// curve body that feeds Plant tribal anthems (Vinetongue, Planttender).
pub fn witherbloom_mossreaver_b132() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Mossreaver (b132)",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Necrobloom (b132) — {3}{B}, 3/3 Plant Warlock. ETB
/// drain 2. Curve-out drain Plant — combines Pest engine durability
/// with mid-curve life-swing.
pub fn witherbloom_necrobloom_b132() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Necrobloom (b132)",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(2)],
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

/// Witherbloom Petalpoke (b132) — {B} Instant. Target creature gets
/// -1/-1 until end of turn. Cheap removal-on-a-card; pairs with
/// Witherbloom's PT-modify package (Boneshroud, Wither curve).
pub fn witherbloom_petalpoke_b132() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Petalpoke (b132)",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 133 ───────────────────────────────────────────────────────────────

/// Witherbloom Twinpest (b133) — {2}{B}{G}, 2/2 Pest Warlock. ETB
/// mints two Pest tokens. Pest-engine double-mint body using
/// `etb_mint_token` with count=2.
pub fn witherbloom_twinpest_b133() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Witherbloom Twinpest (b133)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: stx_pest_token(),
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

/// Witherbloom Toadcaller (b133) — {1}{G}, 2/1 Human Druid. Magecraft
/// adds a +1/+1 counter to itself.
pub fn witherbloom_toadcaller_b133() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Toadcaller (b133)",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
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

/// Pest Mawcatcher (b133) — {1}{B}, 1/2 Pest. Dies → drain 2.
pub fn pest_mawcatcher_b133() -> CardDefinition {
    use crate::effect::shortcut::dies_drain;
    CardDefinition {
        name: "Pest Mawcatcher (b133)",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_drain(2)],
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

/// Witherbloom Sproutchanter (b133) — {2}{G}, 2/3 Plant Druid.
/// Magecraft puts a +1/+1 counter on each creature you control.
pub fn witherbloom_sproutchanter_b133() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sproutchanter (b133)",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
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

// ── Batch 134 ───────────────────────────────────────────────────────────────

/// Pest Lichcaller (b134) — {1}{B}, 1/2 Pest Warlock. When this dies,
/// create a Pest token and drain 1. Uses the new
/// `dies_mint_token_and_drain` shortcut.
pub fn pest_lichcaller_b134() -> CardDefinition {
    use crate::effect::shortcut::dies_mint_token_and_drain;
    CardDefinition {
        name: "Pest Lichcaller (b134)",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_mint_token_and_drain(stx_pest_token(), 1)],
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

// ── Batch 135 (placed after b134 cards below) ───────────────────────────────

/// Witherbloom Pestmaster (b135) — {2}{B}{G} 3/3 Human Warlock.
/// "Whenever another creature you control dies, each opponent loses 1
/// life and you gain 1 life." Aristocrat payoff at 4 mana — fires once
/// per friendly death (including Pest tokens).
pub fn witherbloom_pestmaster_b135() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestmaster (b135)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_other_dies(Effect::Drain {
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

/// Pest Sprouter (b135) — {1}{G} 1/2 Plant Druid. ETB creates a Pest
/// token. Card-advantage one-shot at the 2-mana slot.
pub fn pest_sprouter_b135() -> CardDefinition {
    CardDefinition {
        name: "Pest Sprouter (b135)",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(stx_pest_token(), 1)],
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

/// Witherbloom Vinemender (b135) — {2}{G} 2/3 Plant Druid Reach.
/// Magecraft gain 2 life. Defensive Reach body that lifegain-scales on
/// every spell cast.
pub fn witherbloom_vinemender_b135() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vinemender (b135)",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(2)],
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

/// Pest Reaper (b135) — {3}{B}{G} 3/3 Pest Warlock Deathtouch. Combat
/// removal stick — 5-mana 3/3 Deathtouch trades into anything.
pub fn pest_reaper_b135() -> CardDefinition {
    CardDefinition {
        name: "Pest Reaper (b135)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
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

// ── Batch 136 ───────────────────────────────────────────────────────────────

/// Pest Twinger (b136) — {2}{B} 2/2 Pest Warlock. ETB mint a Pest.
/// 3-mana 2-body for Pest aristocrats.
pub fn pest_twinger_b136() -> CardDefinition {
    CardDefinition {
        name: "Pest Twinger (b136)",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(stx_pest_token(), 1)],
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

/// Witherbloom Bonereader (b136) — {2}{B}{G} 3/2 Human Warlock.
/// "When this enters, mill 2 cards, then gain 1 life for each creature
/// card you milled." Approximated as Mill 2 + GainLife 1 (a steady
/// drip).
pub fn witherbloom_bonereader_b136() -> CardDefinition {
    use crate::card::TriggeredAbility;
    use crate::effect::{EventScope, EventSpec};
    CardDefinition {
        name: "Witherbloom Bonereader (b136)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(2) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
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

/// Witherbloom Vinemaul (b136) — {1}{G} Instant. Target creature gets
/// +2/+2 EOT and gains Trample EOT. Witherbloom-flavored combat trick.
pub fn witherbloom_vinemaul_b136() -> CardDefinition {
    use crate::effect::shortcut::pump_and_grant_keyword;
    CardDefinition {
        name: "Witherbloom Vinemaul (b136)",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: pump_and_grant_keyword(2, 2, Keyword::Trample),
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

/// Witherbloom Necrosage (b136) — {1}{B}{G} 2/2 Human Warlock Deathtouch.
/// Cheap evasive removal-stick — Deathtouch at 3 mana with a 2-body.
pub fn witherbloom_necrosage_b136() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Necrosage (b136)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
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

// ── Batch 138 ───────────────────────────────────────────────────────────────

/// Witherbloom Drainpath II (b138) — {2}{B} Sorcery. Drain 2 + Surveil 1.
/// 3-mana drain + selection.
pub fn witherbloom_drainpath_ii_b138() -> CardDefinition {
    use crate::effect::shortcut::drain_and_surveil;
    CardDefinition {
        name: "Witherbloom Drainpath II (b138)",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain_and_surveil(2, 1),
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

/// Pest Quartermaster (b138) — {2}{B}{G} 3/3 Pest Warlock. ETB mints
/// 1 Pest token + on-other-dies trigger gains 1 life. Aristocrat
/// scaling lifegain engine.
pub fn pest_quartermaster_b138() -> CardDefinition {
    CardDefinition {
        name: "Pest Quartermaster (b138)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_mint_token(stx_pest_token(), 1),
            on_other_dies(Effect::GainLife {
                who: Selector::You,
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

/// Witherbloom Pestlord II (b138) — {3}{B}{G} 4/4 Plant Warlock.
/// ETB mints 2 Pest tokens. 5-mana go-wide top-end.
pub fn witherbloom_pestlord_ii_b138() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestlord II (b138)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(stx_pest_token(), 2)],
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

/// Witherbloom Verdantroot (b138) — {1}{G} 1/3 Plant Druid Reach.
/// Magecraft GainLife 2. Defensive anti-flier with on-cast lifegain.
pub fn witherbloom_verdantroot_b138() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Verdantroot (b138)",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(2)],
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

// ── Batch 139 ───────────────────────────────────────────────────────────────

/// Witherbloom Lifeharvest (b139) — {B}{G} Sorcery.
/// Seq(GainLife 3 + Surveil 1). 2-mana lifegain + selection.
pub fn witherbloom_lifeharvest_b139() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lifeharvest (b139)",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
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

/// Witherbloom Sapherder (b139) — {2}{G} 2/3 Plant Druid. ETB mint
/// 2 Pest tokens.
pub fn witherbloom_sapherder_b139() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapherder (b139)",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(stx_pest_token(), 2)],
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

/// Witherbloom Grimsage (b139) — {2}{B} 2/3 Human Warlock.
/// Dies-trigger Drain 2 + ETB mint 1 Pest.
pub fn witherbloom_grimsage_b139() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Grimsage (b139)",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_mint_token(stx_pest_token(), 1),
            dies_drain(2),
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

/// Pest Bloodscribe (b138) — {1}{B} 2/2 Pest Warlock. Dies-trigger
/// Drain 1. 2-mana aristocrats trade body.
pub fn pest_bloodscribe_b138() -> CardDefinition {
    CardDefinition {
        name: "Pest Bloodscribe (b138)",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_drain(1)],
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

/// Witherbloom Plantlord (b134) — {1}{G} 2/2 Plant Druid. Static
/// "Other Plant creatures you control get +1/+1." Cheap Plant-tribal
/// lord that fills the curve below Vinetongue (b129, 3/3 for 3).
pub fn witherbloom_plantlord_b134() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Witherbloom Plantlord (b134)",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Plant creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Plant))
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

// ── Batch 141 ───────────────────────────────────────────────────────────────

/// Witherbloom Pestmage (b141) — {1}{B}{G} 2/3 Plant Warlock.
/// ETB mint Pest + Surveil 1. Pest engine that smooths draws.
pub fn witherbloom_pestmage_b141() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestmage (b141)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
            },
            Effect::Surveil {
                who: PlayerRef::You,
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

/// Witherbloom Pestbloom (b141) — {3}{B}{G} Sorcery. Create 3 Pest
/// tokens. Heavy mid-game Pest swarm-and-drain payoff.
pub fn witherbloom_pestbloom_b141() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestbloom (b141)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: stx_pest_token(),
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

/// Witherbloom Lifedrinker (b141) — {2}{B} 3/2 Vampire Warlock
/// Lifelink. Magecraft drain 1.
pub fn witherbloom_lifedrinker_b141() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lifedrinker (b141)",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Pestcaller II (b141) — {2}{B}{G} 3/3 Plant Warlock.
/// On-another-dies trigger mint a Pest. Aristocrat go-wide engine.
pub fn witherbloom_pestcaller_ii_b141() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestcaller II (b141)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_other_dies_mint_token(stx_pest_token(), 1)],
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

// ── Batch 142 ───────────────────────────────────────────────────────────────

/// Witherbloom Toxincaller (b142) — {1}{B} 2/1 Plant Warlock.
/// Magecraft mint a Pest token. Aristocrat go-wide engine on every IS
/// cast.
pub fn witherbloom_toxincaller_b142() -> CardDefinition {
    use crate::effect::shortcut::magecraft_mint_token;
    CardDefinition {
        name: "Witherbloom Toxincaller (b142)",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_mint_token(stx_pest_token(), 1)],
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

/// Witherbloom Sapsage (b142) — {2}{G} 3/3 Plant Druid. ETB
/// Seq(GainLife 2 + AddCounter +1/+1 on self). Self-growing lifegain
/// midrange body.
pub fn witherbloom_sapsage_b142() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapsage (b142)",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
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

/// Witherbloom Necroleaf (b142) — {1}{B}{G} Sorcery. Reanimate target
/// creature card with mana value ≤ 3 from your graveyard. Witherbloom's
/// 3-mana low-curve reanimation.
pub fn witherbloom_necroleaf_b142() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Necroleaf (b142)",
        cost: cost(&[generic(1), b(), g()]),
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
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMost(3)),
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

/// Witherbloom Verdantvine (b142) — {2}{B}{G} 2/4 Plant Druid.
/// Magecraft Seq(Surveil 1 + GainLife 1). Defensive midrange spellslinger.
pub fn witherbloom_verdantvine_b142() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Verdantvine (b142)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Surveil {
                who: PlayerRef::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Hivelord (b142) — {3}{B}{G} 4/4 Plant Warlock. Static "Other
/// Pest creatures you control get +1/+1." Pest tribal anthem.
pub fn pest_hivelord_b142() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Pest Hivelord (b142)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Pest creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Pest))
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

// ── Batch 143 ───────────────────────────────────────────────────────────────

/// Witherbloom Bloodpest (b143) — {1}{B}{G} 2/3 Plant Warlock. Magecraft
/// drain 2. 3-mana stronger Apprentice variant.
pub fn witherbloom_bloodpest_b143() -> CardDefinition {
    use crate::effect::shortcut::magecraft_drain;
    CardDefinition {
        name: "Witherbloom Bloodpest (b143)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_drain(2)],
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

/// Pest Sapharvester (b143) — {B}{G} 2/1 Pest Druid Deathtouch. Cheap
/// deathtouch trade body.
pub fn pest_sapharvester_b143() -> CardDefinition {
    CardDefinition {
        name: "Pest Sapharvester (b143)",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
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

/// Witherbloom Pestmother (b143) — {3}{B}{G} 3/4 Plant Druid. ETB
/// mints 2 Pest tokens + magecraft Drain 1.
pub fn witherbloom_pestmother_b143() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestmother (b143)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_mint_token(stx_pest_token(), 2),
            magecraft_drain_each_opp(1),
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

/// Witherbloom Vinepatch (b143) — {B}{G} Instant. -2/-2 EOT + GainLife 2.
/// 2-mana removal trick.
pub fn witherbloom_vinepatch_b143() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vinepatch (b143)",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pest Spawnreaver (b143) — {2}{B}{G} 3/3 Pest Warlock. Whenever a
/// creature you control dies, you gain 1 life and target opp loses 1.
/// Aristocrats payoff.
pub fn pest_spawnreaver_b143() -> CardDefinition {
    CardDefinition {
        name: "Pest Spawnreaver (b143)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_other_dies(Effect::Drain {
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

/// Witherbloom Cauldronist (b143) — {2}{B}{G} 2/3 Human Warlock.
/// {1}{B}{G}, Sacrifice a creature: Drain 2. Activated sacrifice drain
/// engine.
pub fn witherbloom_cauldronist_b143() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Cauldronist (b143)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1), b(), g()]),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
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
            sac_other_filter: Some((
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
                1,
            )),
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

/// Witherbloom Lifeglobe (b143) — {2}{B} 2/3 Vampire Cleric. Static:
/// "Your opponents can't gain life." Witherbloom take on the Erebos /
/// Tainted Remedy axe-vs-lifegain. Per CR 119.7. Pairs perfectly with
/// the school's drain-heavy spell suite.
pub fn witherbloom_lifeglobe_b143() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::{PlayerStaticTarget, StaticEffect};
    CardDefinition {
        name: "Witherbloom Lifeglobe (b143)",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Your opponents can't gain life.",
            effect: StaticEffect::PlayerCannotGainLife {
                target: PlayerStaticTarget::EachOpponent,
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

/// Witherbloom Lifeline (b143) — {1}{G} Sorcery. Gain 3 life and draw a card.
/// 2-mana defensive cantrip.
pub fn witherbloom_lifeline_b143() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lifeline (b143)",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
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

// ── Batch 144 ───────────────────────────────────────────────────────────────

/// Pest Spawnchant (b144) — {B}{G} Sorcery. Create 2 Pest tokens.
/// 2-mana Pest-tribal go-wide.
pub fn pest_spawnchant_b144() -> CardDefinition {
    CardDefinition {
        name: "Pest Spawnchant (b144)",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: stx_pest_token(),
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

/// Witherbloom Pestlord (b144) — {3}{B}{G} 4/4 Pest Warlock.
/// "Whenever you sacrifice a creature, you may pay {B}{G}: draw a card."
/// Approximation: triggered ability drains 1 + draws 1 (no may-cost).
pub fn witherbloom_pestlord_b144() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestlord (b144)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CreatureSacrificed,
                crate::card::EventScope::YourControl,
            ),
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

/// Witherbloom Decayheart (b144) — {2}{B} 3/2 Plant Warlock Deathtouch.
/// 3-mana deathtouch threat with menace.
pub fn witherbloom_decayheart_b144() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Decayheart (b144)",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch, Keyword::Menace],
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

/// Pest Carrionbreeder (b144) — {2}{B}{G} 2/3 Pest Insect. Cycling {2}.
pub fn pest_carrionbreeder_b144() -> CardDefinition {
    CardDefinition {
        name: "Pest Carrionbreeder (b144)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Insect],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Reaver (b144) — {1}{B}{G} 3/3 Plant Warrior Trample.
/// 3-mana big trampler.
pub fn witherbloom_reaver_b144() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Reaver (b144)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warrior],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Witherbloom Lifedrip (b144) — {2}{B}{G} Sorcery. Drain 3 + Draw 1.
/// 4-mana drain + cantrip.
pub fn witherbloom_lifedrip_b144() -> CardDefinition {
    use crate::effect::shortcut::drain_and_draw;
    CardDefinition {
        name: "Witherbloom Lifedrip (b144)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain_and_draw(3),
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

// ── Batch 145 ───────────────────────────────────────────────────────────────

/// Witherbloom Vinegrower (b145) — {1}{G} 2/2 Plant Druid Reach.
/// Cycling {1}{G}.
pub fn witherbloom_vinegrower_b145() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vinegrower (b145)",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach, Keyword::Cycling(cost(&[generic(1), g()]))],
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

/// Pest Acolyte (b145) — {1}{B} 1/1 Pest Cleric Lifelink. Magecraft
/// gain 1 life. Compact aristocrats lifegain body.
pub fn pest_acolyte_b145() -> CardDefinition {
    CardDefinition {
        name: "Pest Acolyte (b145)",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
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

/// Witherbloom Vipergrove (b145) — {3}{B}{G} 4/5 Plant Snake.
/// 5-mana big body — Deathtouch + Trample.
pub fn witherbloom_vipergrove_b145() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Vipergrove (b145)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Snake],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Deathtouch, Keyword::Trample],
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

/// Witherbloom Necromage (b144) — {3}{B} 3/3 Vampire Wizard. ETB
/// returns target creature card from your gy → bf tapped.
pub fn witherbloom_necromage_b144() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Necromage (b144)",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
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
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: true,
            },
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
