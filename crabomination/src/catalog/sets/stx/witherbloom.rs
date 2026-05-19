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
    magecraft, magecraft_drain_each_opp, magecraft_gain_life, magecraft_self_pump, target_filtered,
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
            self_counter_cost_reduction: None,
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
    use crate::card::EventKind;
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
                    self_counter_cost_reduction: None,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            },
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
                    pool: ManaPayload::Colors(vec![Color::Green]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
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
            self_counter_cost_reduction: None,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
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
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
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
