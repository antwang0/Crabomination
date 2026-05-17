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
    EventSpec, Keyword, Predicate, SelectionRequirement, Selector, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::magecraft_drain_each_opp;
use crate::effect::{ManaPayload, PlayerRef, ZoneDest};
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
    }
}
