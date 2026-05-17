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
    }
}
