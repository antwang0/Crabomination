//! Prismari (U/R) college cards from Strixhaven.
//!
//! Prismari's themes are spellslinger payoffs (Magecraft, copy-spell
//! triggers, treasure-style ramp) and creature-makes-token mid-cast
//! synergies. The first wave here covers the basic Apprentice +
//! Pledgemage pair, plus a couple of mono-shape supporting cards.
//! Larger Prismari cards (Magma Opus, Expressive Iteration's siblings)
//! lean on the copy-spell primitive and stay ⏳ until that lands.

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{
    magecraft, magecraft_loot, magecraft_ping_each_opp, magecraft_self_pump, magecraft_treasure,
    target_filtered,
};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{cost, generic, r, u};

// ── Prismari Pledgemage ─────────────────────────────────────────────────────

/// Prismari Pledgemage — {1}{U}{R}, 2/3 Elemental. "Trample, haste."
///
/// Pure stat-line + keyword body. Prismari Pledgemage is the "free
/// vanilla beater" of the Prismari arsenal: a 2/3 trample-haste for
/// {URR}-equivalent costs is solid, and it composes against every
/// pump and copy effect in the college.
pub fn prismari_pledgemage() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pledgemage",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Trample, Keyword::Haste],
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

// ── Prismari Apprentice ─────────────────────────────────────────────────────

/// Prismari Apprentice — {U}{R}, 2/2 Human Wizard.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// choose one — / • Scry 1. / • Prismari Apprentice gets +1/+0 until
/// end of turn."
///
/// ✅ Modal magecraft now wired via `Effect::ChooseMode([Scry 1, +1/+0
/// EOT])`. The engine's CR 700.2b primitive (`pick_trigger_mode` in
/// `game/stack.rs`) asks the controller for the mode at push-time when
/// the trigger lands on the stack — so `AutoDecider` picks mode 0
/// (Scry 1) for the default play pattern, and `ScriptedDecider::new(
/// [DecisionAnswer::Mode(1)])` exercises the +1/+0 branch in tests.
/// The mode pick is a `Decision::ChooseMode { source, num_modes: 2 }`,
/// matching the printed Oracle's "choose one — " wording. Tests:
/// `prismari_apprentice_scry_one_by_default_on_instant_cast`,
/// `prismari_apprentice_can_pump_via_scripted_mode_pick`.
pub fn prismari_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Prismari Apprentice",
        cost: cost(&[u(), r()]),
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
        triggered_abilities: vec![magecraft(Effect::ChooseMode(vec![
            // Mode 0 — Scry 1.
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            // Mode 1 — Prismari Apprentice gets +1/+0 until end of turn.
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
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

// ── Prismari Drakelord (batch 15) ───────────────────────────────────────────

/// Prismari Drakelord — {1}{U}{R}, 2/3 Drake Wizard, Flying.
///
/// Printed Oracle (synthesised): "Flying / Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, this creature gets
/// +1/+1 until end of turn."
///
/// Three-mana evasive Prismari body that snowballs with cast
/// frequency. Single cast turns the Drakelord into a 3/4 flyer; two
/// casts into a 4/5. Same pump shape as Spectacle Mage but with
/// magecraft (instant/sorcery only) instead of prowess.
pub fn prismari_drakelord() -> CardDefinition {
    CardDefinition {
        name: "Prismari Drakelord",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: Selector::This,
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

// ── Prismari Emberseer (batch 15) ───────────────────────────────────────────

/// Prismari Emberseer — {2}{U}{R}, 3/3 Elemental, Flying.
///
/// Printed Oracle (synthesised): "Flying / When this creature enters,
/// it deals 2 damage to each opponent."
///
/// Four-mana finisher with a built-in 2-damage swing to each opp on
/// arrival. Pairs with Magecraft drains (Witherbloom Apprentice
/// extension via Silverquill Stormbringer) for the cumulative drain
/// payoff.
pub fn prismari_emberseer() -> CardDefinition {
    CardDefinition {
        name: "Prismari Emberseer",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
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

// ── Prismari Pyrowriter (batch 15) ──────────────────────────────────────────

/// Prismari Pyrowriter — {U}{R}, 2/2 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, this creature deals 1 damage to
/// any target."
///
/// Two-mana Prismari ping body — every cast becomes a 1-damage shot
/// that closes out games. Same shape as Lorehold Ember-Priest but
/// without the Spirit subtype synergy. Slots into burn-style spell-
/// slinger shells.
pub fn prismari_pyrowriter() -> CardDefinition {
    use crate::card::SelectionRequirement;
    CardDefinition {
        name: "Prismari Pyrowriter",
        cost: cost(&[u(), r()]),
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

// ── Symmetry Sage ───────────────────────────────────────────────────────────

/// Symmetry Sage — {U}, 1/2 Human Wizard.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// Symmetry Sage gets +1/+0 and gains flying until end of turn."
///
/// 🟡 We split the rider into two separate magecraft triggers: one
/// `magecraft_self_pump(+1/+0)` and one grant-flying. They're functionally
/// equivalent to the original `Seq` body — both fire on every magecraft
/// event and both reference the source via `Selector::This`. The split
/// also means the helper is reusable across any future magecraft
/// self-pump creature (e.g. Quandrix's Berta, Symmetry Sage's siblings)
/// without copy-pasting a six-line `Seq`.
pub fn symmetry_sage() -> CardDefinition {
    CardDefinition {
        name: "Symmetry Sage",
        cost: cost(&[u()]),
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
        triggered_abilities: vec![
            magecraft_self_pump(1, 0),
            magecraft(Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
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


// ── Prismari Pyrotechnician (batch 17) ──────────────────────────────────────

/// Prismari Pyrotechnician — {1}{U}{R}, 2/2 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, this creature deals 1 damage to any
/// target."
///
/// Cheap Prismari magecraft body that pings each cast. Pairs with
/// every Magecraft engine — Aziza copy chains, Galvanic Iteration,
/// Symmetry Sage stack — for explosive late-game.
pub fn prismari_pyrotechnician() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyrotechnician",
        cost: cost(&[generic(1), u(), r()]),
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
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature
                .or(SelectionRequirement::Player)
                .or(SelectionRequirement::HasCardType(CardType::Planeswalker))),
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

// ── Prismari Looter (batch 17) ──────────────────────────────────────────────

/// Prismari Looter — {U}{R}, 1/3 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, draw a
/// card, then discard a card."
///
/// Classic UR loot body — a Merfolk Looter shape on a 1/3 body. The
/// loot smooths late-game draws and feeds Magecraft engines via the
/// discard. Pairs with Plargg / Looter's-style discard payoffs.
pub fn prismari_looter() -> CardDefinition {
    CardDefinition {
        name: "Prismari Looter",
        cost: cost(&[u(), r()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
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

// ── Prismari Chromaticist (batch 17) ────────────────────────────────────────

/// Prismari Chromaticist — {2}{U}{R}, 3/3 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// Treasure token. (It's an artifact with '{T}, Sacrifice this artifact:
/// Add one mana of any color.')"
///
/// Mid-curve Prismari ramp + body. Pairs with Prismari Treasurewright
/// for double-Treasure ETB chains. The Treasure goes through the
/// existing `Effect::CreateToken` + `treasure_token` plumbing.
pub fn prismari_chromaticist() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Chromaticist",
        cost: cost(&[generic(2), u(), r()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
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

// ── Prismari Drakeward (batch 17) ───────────────────────────────────────────

/// Prismari Drakeward — {3}{U}{R}, 4/4 Drake, Flying.
///
/// Printed Oracle (synthesised): "Flying / When this creature enters,
/// it deals 2 damage to each opponent."
///
/// Five-mana 4/4 flier with built-in 2-damage drain-equivalent ETB.
/// Pairs naturally with Prismari Pyrotechnician's spell-pings for
/// rapid finish.
pub fn prismari_drakeward() -> CardDefinition {
    CardDefinition {
        name: "Prismari Drakeward",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
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

// ── Prismari Spellsmith (batch 18) ─────────────────────────────────────────

/// Prismari Spellsmith — {1}{U}{R}, 2/2 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// Treasure token. (It's an artifact with '{T}, Sacrifice this artifact:
/// Add one mana of any color.')"
///
/// Three-mana ramp + body — drops a 2/2 plus an immediately-usable
/// Treasure. Same template as Spectacular Skywhale but on a {U}{R}
/// 2/2 body instead of a 1/4 flyer. Pairs with Prismari Treasurewright
/// for double-Treasure ETB chains.
pub fn prismari_spellsmith() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Spellsmith",
        cost: cost(&[generic(1), u(), r()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
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

// ── Prismari Storm-Caller (batch 18) ───────────────────────────────────────

/// Prismari Storm-Caller — {2}{U}{R}, 3/2 Elemental Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, draw a card, then discard a card."
///
/// Looter-tron Prismari magecraft body — every cast becomes a loot.
/// Fuels graveyard recursion (Pillardrop Rescuer, Lorehold Excavation)
/// and feeds Tablet of Discovery-style discard payoffs.
pub fn prismari_storm_caller() -> CardDefinition {
    CardDefinition {
        name: "Prismari Storm-Caller",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Prismari Ignite-Apprentice (batch 18) ──────────────────────────────────

/// Prismari Ignite-Apprentice — {1}{R}, 2/1 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, it deals
/// 1 damage to any target."
///
/// Mini-Sparkmage Apprentice on a {1}{R} frame. The ETB ping closes
/// out random small creatures (2/1 trade into 1/1) or shaves a final
/// life off a planeswalker. Distinct from extras.rs's `prismari_sparkmage`
/// (a 2/3 Magecraft ping body) — different stat-line and trigger.
pub fn prismari_ignite_apprentice() -> CardDefinition {
    use crate::card::SelectionRequirement;
    CardDefinition {
        name: "Prismari Ignite-Apprentice",
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

// ── Prismari Volley (batch 18) ─────────────────────────────────────────────

/// Prismari Volley — {2}{R} Instant.
///
/// Printed Oracle (synthesised): "Prismari Volley deals 3 damage to
/// target creature or planeswalker. Draw a card."
///
/// Three-mana Prismari removal cantrip — a creature/planeswalker-only
/// burn with built-in card advantage. Strictly weaker than Lightning
/// Bolt on the body side (no player damage) but trades up via the
/// draw. Pairs aggressively with magecraft + opus payoff bodies.
pub fn prismari_volley() -> CardDefinition {
    use crate::card::SelectionRequirement;
    CardDefinition {
        name: "Prismari Volley",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
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

// ── Prismari Stormcaster (batch 19) ────────────────────────────────────────

/// Prismari Stormcaster — {3}{U}{R}, 3/3 Djinn Wizard, Flying.
///
/// Printed Oracle (synthesised): "Flying / Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, draw a card, then discard
/// a card."
///
/// Looter-tron-on-a-flier — 5-mana 3/3 evasive body that loots every
/// cast. Stronger sibling to Prismari Storm-Caller (1 power, no
/// flying) on a heavier curve. Pairs with Goldspan Dragon-style
/// big-creature recursion via the constant graveyard refill.
pub fn prismari_stormcaster() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormcaster",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Prismari Sparkmaster (batch 19) ────────────────────────────────────────

/// Prismari Sparkmaster — {2}{U}{R}, 2/2 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, this creature gets +1/+0 until
/// end of turn."
///
/// Cheap magecraft self-pump — every cast turns the Sparkmaster into
/// a bigger attacker for the turn. Mirror of Eccentric Apprentice on
/// a sturdier 2/2 frame at the 4-mana slot.
pub fn prismari_sparkmaster() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkmaster",
        cost: cost(&[generic(2), u(), r()]),
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

// ── Prismari Ember-Channeler (batch 19) ────────────────────────────────────

/// Prismari Ember-Channeler — {U}{R}, 1/2 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, this creature deals 1 damage to
/// any target."
///
/// 2-mana Lorehold Apprentice mirror at the {U}{R} slot — every cast
/// pings 1 damage to any target. Functions as a budget-Reverberator
/// (2 mana vs 4) at half the damage; cheap, fragile, but compounds.
pub fn prismari_ember_channeler() -> CardDefinition {
    CardDefinition {
        name: "Prismari Ember-Channeler",
        cost: cost(&[u(), r()]),
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

// ── Prismari Alchemist (batch 19+) ─────────────────────────────────────────

/// Prismari Alchemist — {2}{U}{R}, 2/3 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, create a Treasure token."
///
/// 4-mana magecraft Treasure-mint body. Each cast feeds the ramp
/// chain — combo with Goldspan Dragon (Treasure on attack) and
/// Galazeth Prismari for explosive midgame mana. Slot into Prismari
/// big-spell shells (Magma Opus, Crackle with Power).
pub fn prismari_alchemist() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Alchemist",
        cost: cost(&[generic(2), u(), r()]),
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
        triggered_abilities: vec![magecraft(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: treasure_token(),
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

// ── Prismari Cantrip (batch 19+) ───────────────────────────────────────────

/// Prismari Cantrip — {U}{R} Instant.
///
/// Printed Oracle (synthesised): "Prismari Cantrip deals 1 damage to
/// target creature. Draw a card."
///
/// 2-mana cheap cantrip-burn. Kills a 1-toughness creature for free
/// (replaces itself) or shaves planeswalker loyalty. Bread-and-butter
/// magecraft enabler in Prismari spell-heavy shells.
pub fn prismari_cantrip() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cantrip",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(1),
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

// ── Prismari Flarespark (batch 19) ─────────────────────────────────────────

/// Prismari Flarespark — {1}{U}{R} Instant.
///
/// Printed Oracle (synthesised): "Prismari Flarespark deals 2 damage
/// to any target. Draw a card."
///
/// 3-mana instant burn cantrip. Mirror of Prismari Volley at lower
/// damage (2 vs 3) but with the wider "any target" range. Same
/// post-cast card draw, so it's a strict tempo trade — replace itself
/// while removing a 2-toughness creature or punching a planeswalker.
pub fn prismari_flarespark() -> CardDefinition {
    CardDefinition {
        name: "Prismari Flarespark",
        cost: cost(&[generic(1), u(), r()]),
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

// ── Prismari Cascade Volley (batch 20) ─────────────────────────────────────

/// Prismari Cascade Volley — {2}{R} Sorcery.
///
/// Printed Oracle (synthesised): "Prismari Cascade Volley deals 3 damage
/// to any target and 1 damage to each creature an opponent controls."
///
/// 3-mana 3-damage + 1-damage-to-each-opp-creature wrath at the burn
/// slot. Anti-token-go-wide tech that also kills a small problem
/// creature outright.
pub fn prismari_cascade_volley() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cascade Volley",
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
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
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

// ── Prismari Initiate (batch 20) ───────────────────────────────────────────

/// Prismari Initiate — {1}{R}, 2/2 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, this creature deals 1 damage to any
/// target."
///
/// 2-mana magecraft ping body — each IS cast pings any target for 1.
/// Strict-better-than-Mascot-Exhibition at this slot since it removes
/// 1-toughness creatures or chips planeswalkers.
pub fn prismari_initiate() -> CardDefinition {
    CardDefinition {
        name: "Prismari Initiate",
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

// ── Prismari Spellbinder (batch 20) ────────────────────────────────────────

/// Prismari Spellbinder — {3}{U}{R}, 3/4 Djinn Wizard with Flying.
///
/// Printed Oracle (synthesised): "Flying. When this creature enters,
/// copy target instant or sorcery spell you control. You may choose new
/// targets for the copy."
///
/// 5-mana ETB-copy flier — copies an instant/sorcery cast for value.
/// Sits perfectly atop the Magma Opus / Crackle with Power chain for
/// doubled damage / token output. Functional cousin of Snapcaster Mage
/// reshaped as an ETB copy at the {U}{R} slot.
pub fn prismari_spellbinder() -> CardDefinition {
    CardDefinition {
        name: "Prismari Spellbinder",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CopySpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(
                            SelectionRequirement::HasCardType(CardType::Instant)
                                .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                        )
                        .and(SelectionRequirement::ControlledByYou),
                ),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Prismari Treasurer (batch 20) ──────────────────────────────────────────

/// Prismari Treasurer — {1}{U}, 1/2 Merfolk Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// Treasure token."
///
/// 2-mana 1/2 + Treasure ETB — ramps {1} of any color for the next
/// turn. Slots into Prismari big-spell shells.
pub fn prismari_treasurer() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Treasurer",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
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
                definition: treasure_token(),
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

// ── Prismari Embershaper (batch 20) ────────────────────────────────────────

/// Prismari Embershaper — {U}{R}, 2/1 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, you may discard a card. If you do, draw
/// a card."
///
/// 2-mana magecraft loot body — every IS cast offers a discard+draw
/// rummage. Combos with discard-payoffs (Honor of the Pure, Madness)
/// and graveyard recursion (Lorehold) for free loot value.
pub fn prismari_embershaper() -> CardDefinition {
    CardDefinition {
        name: "Prismari Embershaper",
        cost: cost(&[u(), r()]),
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
        triggered_abilities: vec![magecraft(Effect::MayDo {
            description: "Discard a card and draw a card".to_string(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
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

// ── Prismari Sparkforge (batch 21) ─────────────────────────────────────────

/// Prismari Sparkforge — {2}{U}{R}, 3/3 Elemental with Haste.
///
/// Printed Oracle (synthesised): "Haste. When this creature enters, create
/// a Treasure token."
///
/// 4-mana hasty 3/3 with built-in mana ramp. Trades and replaces its
/// initial spend, accelerating into bigger Prismari finishers.
pub fn prismari_sparkforge() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Sparkforge",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
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

// ── Prismari Mindwave (batch 21) ───────────────────────────────────────────

/// Prismari Mindwave — {2}{U} Instant.
///
/// Printed Oracle (synthesised): "Draw two cards, then discard a card."
///
/// 3-mana net +1 card with looter quality. Filters dead draws while
/// digging through the deck for Prismari finishers. Functionally same
/// effect as Brainstorm-but-3-mana with no shuffle interaction.
pub fn prismari_mindwave() -> CardDefinition {
    CardDefinition {
        name: "Prismari Mindwave",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
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

// ── Prismari Pyrocrafter (batch 21) ────────────────────────────────────────

/// Prismari Pyrocrafter — {2}{R}, 2/2 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, it deals 1
/// damage to each opponent. Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, this creature gets +1/+0 until end of turn."
///
/// 3-mana ETB ping + magecraft self-pump. Scales aggressively through
/// the mid-game as the spell count climbs.
pub fn prismari_pyrocrafter() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyrocrafter",
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
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
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

// ── Prismari Stormspire (batch 21) ─────────────────────────────────────────

/// Prismari Stormspire — {4}{U}{R}, 4/4 Djinn Wizard with Flying.
///
/// Printed Oracle (synthesised): "Flying. When this creature enters, draw
/// two cards."
///
/// 6-mana finisher Sphinx body — flying 4/4 + 2-card draw on ETB.
/// Race-breaking top-end that immediately rebuilds the hand. Slightly
/// undercosted Mulldrifter on a body.
pub fn prismari_stormspire() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormspire",
        cost: cost(&[generic(4), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
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

// ── Prismari batch 22 ──────────────────────────────────────────────────────

/// Prismari Sparkforger — {2}{U}{R}, 2/4 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, target creature you control gets +1/+0
/// and gains haste until end of turn."
///
/// 4-mana magecraft team-pumper. Trigger flexes between aggressive haste
/// for a sleeping creature or +1/+0 on a hasty attacker for a 1-damage
/// boost.
pub fn prismari_spellforger_b22() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkforger",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::TriggerSource,
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
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

/// Prismari Volleyfire — {3}{R} Sorcery.
///
/// Printed Oracle (synthesised): "Prismari Volleyfire deals 4 damage to
/// target creature or planeswalker. Mint a Treasure token."
///
/// 4-mana hard removal + ramp. Trades a card for a 4-damage shot and
/// rebuilds a mana pip on the same resolution. Combos with Big Score-
/// style Treasure synergies in any Prismari shell.
pub fn prismari_volleyfire() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Volleyfire",
        cost: cost(&[generic(3), r()]),
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
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(4),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
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

/// Prismari Spell-Shaper — {U}{R}, 1/3 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, scry 1, then draw a card."
///
/// 2-mana magecraft scry-cantrip. Smooths every draw on every IS cast —
/// the centerpiece of a Prismari spell-control deck.
pub fn prismari_spell_shaper() -> CardDefinition {
    CardDefinition {
        name: "Prismari Spell-Shaper",
        cost: cost(&[u(), r()]),
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
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
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

/// Prismari Stormgaze — {2}{U}{R} Instant.
///
/// Printed Oracle (synthesised): "Draw two cards. Then discard a card.
/// Prismari Stormgaze deals 1 damage to any target."
///
/// 4-mana looter + ping. Three modes: ping creature/PW for 1, soft-loot
/// for 2-keep-1, all on one spell.
pub fn prismari_stormgaze() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormgaze",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
            },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
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

/// Prismari Vortexweaver — {3}{U}{R}, 3/4 Elemental Wizard with Flying.
///
/// Printed Oracle (synthesised): "Flying. When this creature enters,
/// copy target instant or sorcery spell you control. You may choose
/// new targets for the copy."
///
/// 5-mana finisher Wizard with a built-in Galvanic Iteration on
/// arrival. The ETB copy-spell only fires if you cast it after another
/// IS spell — but in a Prismari shell that's most turns.
pub fn prismari_vortexweaver() -> CardDefinition {
    CardDefinition {
        name: "Prismari Vortexweaver",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CopySpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack.and(
                        SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    ),
                ),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Prismari Quickfire (batch 21) ──────────────────────────────────────────

/// Prismari Quickfire — {R} Instant.
///
/// Printed Oracle (synthesised): "Prismari Quickfire deals 2 damage to
/// target creature."
///
/// 1-mana 2-damage burn — efficient creature removal at the curve-1 slot.
/// Triggers magecraft for the cheapest possible spell cost. Same shape
/// as Burst Lightning at the {R} slot.
pub fn prismari_quickfire() -> CardDefinition {
    CardDefinition {
        name: "Prismari Quickfire",
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 23: 5 new Prismari cards ─────────────────────

/// Prismari Treasurer-Surge — {3}{U}{R}, 4/3 Elemental Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, create two
/// Treasure tokens. Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature gets +1/+0 until end of turn."
///
/// 5-mana ramp engine + cast scaling. Two Treasure tokens on ETB means the
/// next 2 spells cast for free in terms of mana sources, and each cast
/// pumps the body for the alpha-strike turn.
pub fn prismari_treasurer_surge() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Treasurer-Surge",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: treasure_token(),
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

/// Prismari Conflagration — {3}{R}, sorcery.
///
/// Printed Oracle (synthesised): "Prismari Conflagration deals 3 damage to
/// each creature."
///
/// 4-mana board sweep — Anger of the Gods at the {3}{R} slot without the
/// exile rider.
pub fn prismari_pyreburst() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyreburst",
        cost: cost(&[generic(3), r()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Vorthos — {2}{U}{R}, 3/3 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, draw a card,
/// then discard a card. If you discarded an instant or sorcery card, this
/// creature deals 2 damage to any target."
///
/// 4-mana looter that converts a discarded IS card into 2 burn damage —
/// rewards Prismari's IS-discard payoff theme.
pub fn prismari_vorthos() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Prismari Vorthos",
        cost: cost(&[generic(2), u(), r()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CardsDiscardedThisEffect,
                        Value::Const(1),
                    ),
                    then: Box::new(Effect::DealDamage {
                        to: target_filtered(
                            SelectionRequirement::Creature
                                .or(SelectionRequirement::Player)
                                .or(SelectionRequirement::Planeswalker),
                        ),
                        amount: Value::Const(2),
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

/// Prismari Cinderspark — {R}, instant.
///
/// Printed Oracle (synthesised): "Prismari Cinderspark deals 1 damage to
/// any target. Scry 1."
///
/// 1-mana ping + scry — the Prismari cantrip slot. Combines burn flexibility
/// with deck smoothing and feeds magecraft triggers.
pub fn prismari_cinderspark() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cinderspark",
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

/// Prismari Tempo Adept — {U}{R}, 2/2 Human Wizard Prowess.
///
/// Printed Oracle (synthesised): "Prowess (Whenever you cast a noncreature
/// spell, this creature gets +1/+1 until end of turn.) When this creature
/// enters, you may discard a card. If you do, draw a card."
///
/// 2-mana prowess body with a built-in optional loot ETB — slots into any
/// IS-heavy Prismari shell and triggers magecraft chains.
pub fn prismari_tempo_adept() -> CardDefinition {
    use crate::effect::shortcut::prowess;
    CardDefinition {
        name: "Prismari Tempo Adept",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Prowess],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "discard a card to draw a card".to_string(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Discard {
                            who: Selector::You,
                            amount: Value::Const(1),
                            random: false,
                        },
                        Effect::Draw {
                            who: Selector::You,
                            amount: Value::Const(1),
                        },
                    ])),
                },
            },
            prowess(),
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

// ── Push (modern_decks) batch 24+: 2 more Prismari cards ───────────────────

/// Prismari Hotburst — {1}{R}, instant.
///
/// Printed Oracle (synthesised): "Prismari Hotburst deals 2 damage to
/// any target. Treasure token."
///
/// 2-mana cheap burn + Treasure ramp. Same shape as Galvanic Iteration's
/// supporting role — gets a 2-damage Shock-equivalent and refunds part of
/// the cost.
pub fn prismari_hotburst() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Hotburst",
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
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
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

/// Prismari Magmaspark — {U}{R}, 1/3 Elemental Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, it deals 1
/// damage to any target. Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, this creature gets +1/+0 until end of turn."
///
/// 2-mana ETB ping + magecraft scaling. Slots into the Prismari curve
/// at the 2-mana spot.
pub fn prismari_magmaspark() -> CardDefinition {
    use crate::effect::shortcut::magecraft_self_pump;
    CardDefinition {
        name: "Prismari Magmaspark",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::Player)
                            .or(SelectionRequirement::Planeswalker),
                    ),
                    amount: Value::Const(1),
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

// ── Push (modern_decks) batch 24: 5 new Prismari cards ─────────────────────

/// Prismari Mindkindler — {U}{R}, 1/2 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, target creature can't block this turn."
///
/// 2-mana Prismari evasion enabler — every cast unblocks an attacker.
/// Pairs with Sparkmage Apprentice / Prismari Sparkbright for chained
/// damage.
pub fn prismari_mindkindler() -> CardDefinition {
    CardDefinition {
        name: "Prismari Mindkindler",
        cost: cost(&[u(), r()]),
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
        triggered_abilities: vec![magecraft(Effect::Tap {
            what: target_filtered(SelectionRequirement::Creature),
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

/// Prismari Embergem — {2}{R}, sorcery.
///
/// Printed Oracle (synthesised): "Prismari Embergem deals 4 damage to
/// target creature. Create a Treasure token."
///
/// 3-mana headline burn + ramp — kills a 4-toughness body and refunds
/// part of the mana into a Treasure.
pub fn prismari_embergem() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Embergem",
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
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
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

/// Prismari Pyromancer — {2}{U}{R}, 3/2 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, deal 2
/// damage to any target. Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, you may discard a card. If you do, draw a
/// card."
///
/// 4-mana ETB burn + magecraft loot — Prismari's signature value engine
/// at a moderate cost.
pub fn prismari_pyromancer() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyromancer",
        cost: cost(&[generic(2), u(), r()]),
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
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::Player)
                            .or(SelectionRequirement::Planeswalker),
                    ),
                    amount: Value::Const(2),
                },
            },
            magecraft(Effect::MayDo {
                description: "discard a card to draw one".to_string(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                ])),
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

/// Prismari Spitfire — {3}{R}, 3/3 Elemental Haste.
///
/// Printed Oracle (synthesised): "Haste. When this creature enters, it
/// deals 2 damage to any target."
///
/// 4-mana ETB-burn finisher — a Flametongue-Kavu-on-a-haster shape
/// (haste + ETB damage). Pure tempo finisher in the Prismari burn shell.
pub fn prismari_spitfire() -> CardDefinition {
    CardDefinition {
        name: "Prismari Spitfire",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Wildform — {U}{R}, instant.
///
/// Printed Oracle (synthesised): "Target creature gets +2/+1 and gains
/// haste until end of turn. Draw a card."
///
/// 2-mana combat trick + cantrip in Prismari colors — pump + haste makes
/// for surprise lethal lines off freshly-cast creatures.
pub fn prismari_wildform() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Prismari Wildform",
        cost: cost(&[u(), r()]),
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
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
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

/// Prismari Sparkbright — {1}{R}, 2/1 Elemental Wizard Haste.
///
/// Printed Oracle (synthesised): "Haste. Whenever this creature attacks,
/// it deals 1 damage to any target."
///
/// 2-mana hasty 2/1 with built-in ping on every attack. Cleanly threatens
/// PWs (knocks 1 loyalty) and answers X/1s.
pub fn prismari_sparkbright() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkbright",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
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


// ── Push (modern_decks) batch 24++: 1 more Prismari card ───────────────────

/// Prismari Drakeforge — {2}{U}{R}, 2/3 Drake Wizard Flying.
///
/// Printed Oracle (synthesised): "Flying. When this creature enters,
/// create a Treasure token. Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, this creature gets +1/+0 until end of turn."
///
/// 4-mana evasive Prismari engine — ETB ramp + per-cast self-pump. Scales
/// aggressively in spell-heavy shells.
pub fn prismari_drakeforge() -> CardDefinition {
    use crate::effect::shortcut::magecraft_self_pump;
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Drakeforge",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: treasure_token(),
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

// ── Push (modern_decks) batch 25: 5 more Prismari cards ────────────────────
//
// Continuing Prismari (U/R) buildout: 3 new creatures + 2 spells using
// existing Treasure / magecraft / loot / damage primitives. No new engine
// features required.

/// Prismari Sparkdrake — {3}{U}{R}, 3/3 Drake Flying Haste.
///
/// Printed Oracle (synthesised): "Flying, haste."
///
/// 5-mana 3/3 flying haste — immediate evasive pressure. Slots into Prismari
/// aggro shells looking for a finisher that ignores ground stalls.
pub fn prismari_sparkdrake() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkdrake",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
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

/// Prismari Lavalifter — {2}{R}, 3/2 Elemental.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// Treasure token."
///
/// 3-mana 3/2 + Treasure-ramp. Net cost: 2-mana 3/2 with a future-turn
/// {1} discount. Slots into any artifact-aware or treasure-payoff shell.
pub fn prismari_lavalifter() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Lavalifter",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
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
                count: Value::Const(1),
                definition: treasure_token(),
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

/// Prismari Spelltheorist — {1}{U}{R}, 2/2 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, draw a card,
/// then discard a card."
///
/// 3-mana looter ETB — replaces itself in card quality and fuels the
/// graveyard for Flashback / Hofri / Lorehold recursion shells.
pub fn prismari_spelltheorist() -> CardDefinition {
    CardDefinition {
        name: "Prismari Spelltheorist",
        cost: cost(&[generic(1), u(), r()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
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

/// Prismari Stormwriter — {2}{U}{R}, instant.
///
/// Printed Oracle (synthesised): "Prismari Stormwriter deals 3 damage to
/// target creature. Draw a card."
///
/// 4-mana burn-and-cantrip — kills 3-toughness creatures + replaces itself
/// in hand. Bread-and-butter UR card advantage.
pub fn prismari_stormwriter() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormwriter",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
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

/// Prismari Igniter — {1}{R}, 2/1 Elemental.
///
/// Printed Oracle (synthesised): "Haste. Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, this creature deals 1 damage to any
/// target."
///
/// 2-mana haste-pinger with per-cast burn. The Magma Hammer template at
/// the {1}{R} slot — every IS spell becomes a free 1-damage shot.
pub fn prismari_igniter() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Prismari Igniter",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
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

// ── Push (modern_decks) batch 28: 5 more Prismari cards ────────────────────
//
// Continuing Prismari (U/R) buildout: 5 new cards using existing primitives.
// No new engine features required.

/// Prismari Embershaper-Wizard — {2}{U}{R}, 2/3 Djinn Wizard Flying.
///
/// Printed Oracle (synthesised): "Flying. When this creature enters, create
/// a Treasure token and discard a card, then draw a card."
///
/// 4-mana evasive ramp + loot. Treasure + loot in one ETB makes for a
/// strong tempo play in any UR spell-heavy shell.
pub fn prismari_embershaper_wizard() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Embershaper-Wizard",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: treasure_token(),
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
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

/// Prismari Magmaboon — {2}{R}, sorcery.
///
/// Printed Oracle (synthesised): "Prismari Magmaboon deals 3 damage to
/// target creature. Create a Treasure token."
///
/// 3-mana burn + ramp combo. Smaller cousin of Prismari Embergem trading
/// 1 damage for 1 less mana.
pub fn prismari_magmaboon() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Magmaboon",
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
                amount: Value::Const(3),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
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

/// Prismari Tideburst — {U}{R}, instant.
///
/// Printed Oracle (synthesised): "Counter target spell unless its
/// controller pays {2}. Scry 1."
///
/// 2-mana flexible tempo counterspell + smoothing. Mana Leak template
/// with scry rider.
pub fn prismari_tideburst() -> CardDefinition {
    CardDefinition {
        name: "Prismari Tideburst",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CounterUnlessPaid {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                mana_cost: cost(&[generic(2)]),
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

/// Prismari Tempest-Caller — {1}{U}{R}, 2/2 Elemental Wizard Flying.
///
/// Printed Oracle (synthesised): "Flying. Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, this creature gets +1/+0 until end of
/// turn."
///
/// 3-mana evasive prowess-on-cast body. Same shape as Spectacle Mage but
/// with magecraft trigger rather than prowess for the IS-only payoff.
pub fn prismari_tempest_caller() -> CardDefinition {
    CardDefinition {
        name: "Prismari Tempest-Caller",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
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

/// Prismari Pyresurge — {3}{R}, sorcery.
///
/// Printed Oracle (synthesised): "Prismari Pyresurge deals 3 damage to any
/// target. Draw a card."
///
/// 4-mana flexible damage + cantrip. Trades up against most 3-toughness
/// creatures while keeping card-neutral.
pub fn prismari_pyresurge_b28() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyresurge",
        cost: cost(&[generic(3), r()]),
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

// ── Batch 30: 5 new Prismari cards ─────────────────────────────────────────

/// Prismari Sparksong — {2}{U}{R}, instant.
///
/// Synthesised Oracle: "Prismari Sparksong deals 3 damage to target creature
/// or planeswalker. Draw a card."
///
/// 4-mana burn + cantrip — fuels Magecraft chains.
pub fn prismari_sparksong() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparksong",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
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

/// Prismari Glasscaster — {U}{R}, 2/2 Elemental Wizard.
///
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature gets +1/+1 until end of turn."
pub fn prismari_glasscaster() -> CardDefinition {
    CardDefinition {
        name: "Prismari Glasscaster",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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

/// Prismari Treasurewright — {2}{R}, 2/3 Djinn Wizard.
///
/// Synthesised Oracle: "When this creature enters, create a Treasure token.
/// Magecraft — Whenever you cast or copy an instant or sorcery spell, scry 1."
pub fn prismari_treasurewright_b30() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Treasurewright B30",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Wizard],
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
                    definition: treasure_token(),
                },
            },
            magecraft(Effect::Scry {
                who: PlayerRef::You,
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

/// Prismari Tideforger — {1}{U}, 2/1 Merfolk Wizard, Flash.
///
/// Synthesised Oracle: "Flash. Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, this creature gets +1/+0 until end of turn."
///
/// Flash-in surprise blocker + magecraft self-pump finisher.
pub fn prismari_tideforger() -> CardDefinition {
    CardDefinition {
        name: "Prismari Tideforger",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash],
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

/// Prismari Splashcaster — {2}{U}{R}, sorcery.
///
/// Synthesised Oracle: "Prismari Splashcaster deals 2 damage to any target
/// and 2 damage to each opponent. Create a Treasure token."
pub fn prismari_splashcaster() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Splashcaster",
        cost: cost(&[generic(2), u(), r()]),
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
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
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

// ── Batch 32 (modern_decks) — Prismari expansion ────────────────────────────

/// Prismari Embertongue — {1}{R}, 2/1 Human Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature deals 1 damage to each opponent."
pub fn prismari_embertongue() -> CardDefinition {
    CardDefinition {
        name: "Prismari Embertongue",
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

/// Prismari Treasurewright — {U}{R}, 2/2 Human Artificer.
/// Synthesised Oracle: "When this creature enters, create a Treasure token."
pub fn prismari_treasurewright_b32() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Treasurewright (b32)",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
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
                definition: treasure_token(),
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

/// Prismari Sparkpainter — {2}{U}{R}, 3/3 Elemental Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature gets +1/+0 until end of turn and you may
/// draw a card. If you do, discard a card." (Loot rider attached via the
/// magecraft trigger.)
pub fn prismari_sparkpainter() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkpainter",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::MayDo {
                description: "Loot 1".to_string(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                ])),
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

/// Prismari Burning Lesson — {U}{R}, sorcery.
/// Synthesised Oracle: "Prismari Burning Lesson deals 3 damage to any
/// target. Scry 1."
pub fn prismari_burning_lesson() -> CardDefinition {
    CardDefinition {
        name: "Prismari Burning Lesson",
        cost: cost(&[u(), r()]),
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

// ── Batch 33: 3 new Prismari cards ────────────────────────────────────

/// Prismari Sparkscribe — {1}{U}{R}, 2/2 Human Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, scry 1."
pub fn prismari_sparkscribe() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkscribe",
        cost: cost(&[generic(1), u(), r()]),
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

/// Prismari Ember-Adept — {2}{U}{R}, 3/3 Elemental Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, this creature deals 1 damage to each
/// opponent."
pub fn prismari_ember_adept() -> CardDefinition {
    CardDefinition {
        name: "Prismari Ember-Adept",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Sparkflare — {2}{R}, Instant.
/// Synthesised Oracle: "Prismari Sparkflare deals 3 damage to any
/// target."
pub fn prismari_sparkflare() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkflare",
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Flameforger — {3}{R}, 3/3 Djinn Wizard Haste.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature gets +2/+0 until end of turn."
pub fn prismari_flameforger() -> CardDefinition {
    CardDefinition {
        name: "Prismari Flameforger",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(2, 0)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 34: Prismari cards ────────────────────────────────────────────────

/// Prismari Stormfront — {3}{U}{R}, Sorcery.
/// Synthesised Oracle: "Prismari Stormfront deals 4 damage to target
/// creature. Draw a card."
pub fn prismari_stormfront() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormfront",
        cost: cost(&[generic(3), u(), r()]),
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

/// Prismari Eruption Mage — {2}{U}{R}, 3/3 Elemental Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature deals 2 damage to any target."
pub fn prismari_eruption_mage() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Prismari Eruption Mage",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
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

/// Prismari Flamescribe — {1}{U}{R}, 2/2 Human Wizard.
/// Synthesised Oracle: "When this creature enters, draw a card, then
/// discard a card."
pub fn prismari_flamescribe() -> CardDefinition {
    CardDefinition {
        name: "Prismari Flamescribe",
        cost: cost(&[generic(1), u(), r()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
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

/// Prismari Sparkriot — {1}{R}, Instant.
/// Synthesised Oracle: "Prismari Sparkriot deals 3 damage to target
/// creature. Draw a card."
pub fn prismari_sparkriot() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkriot",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
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

/// Prismari Pyrosage — {3}{R}, 3/2 Human Wizard with Haste.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature deals 1 damage to each opponent."
pub fn prismari_pyrosage() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyrosage",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 35: Prismari cards ────────────────────────────────────────────────

/// Prismari Spellforge — {2}{U}{R}, 3/3 Elemental Wizard.
/// Synthesised Oracle: "When this creature enters, deal 2 damage to any
/// target. Magecraft — Loot (Draw 1, discard 1)."
pub fn prismari_spellforge() -> CardDefinition {
    CardDefinition {
        name: "Prismari Spellforge",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
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
                effect: Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::Player)
                            .or(SelectionRequirement::Planeswalker),
                    ),
                    amount: Value::Const(2),
                },
            },
            magecraft_loot(),
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

/// Prismari Pyromage — {R}{U}, 2/1 Human Wizard.
/// Synthesised Oracle: "Magecraft — This creature deals 1 damage to any
/// target."
pub fn prismari_b35_pyromage() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Prismari Pyromage II",
        cost: cost(&[r(), u()]),
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

/// Prismari Stormforge — {3}{U}{R}, Sorcery.
/// Synthesised Oracle: "This deals 3 damage to target creature. Draw 2
/// cards."
pub fn prismari_stormforge() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormforge",
        cost: cost(&[generic(3), u(), r()]),
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
            Effect::Draw {
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

/// Prismari Mirror Mage — {2}{U}{R}, 2/3 Elemental Wizard.
/// Synthesised Oracle: "Magecraft — This creature gets +1/+1 EOT."
pub fn prismari_mirror_mage() -> CardDefinition {
    CardDefinition {
        name: "Prismari Mirror Mage",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
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

// ── Batch 37: more Prismari cards ───────────────────────────────────────────

/// Prismari Sparkmage — {2}{R}, 2/2 Human Wizard.
/// Synthesised Oracle: "When this creature enters, deal 2 damage to any
/// target. Magecraft — ping 1 each opponent."
pub fn prismari_sparkmage_v2() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkmage II",
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
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::Player)
                            .or(SelectionRequirement::Planeswalker),
                    ),
                    amount: Value::Const(2),
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

/// Prismari Eddy — {U}, Instant.
/// Synthesised Oracle: "Draw a card. Scry 1."
pub fn prismari_eddy() -> CardDefinition {
    CardDefinition {
        name: "Prismari Eddy",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 38: more Prismari cards ───────────────────────────────────────────

/// Prismari Dazzler — {1}{U}{R}, 2/2 Elemental Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, this creature deals 1 damage to any target."
pub fn prismari_dazzler() -> CardDefinition {
    CardDefinition {
        name: "Prismari Dazzler",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::magecraft_ping_any(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Cinderpoet — {2}{U}{R}, 3/2 Elemental Wizard.
/// Synthesised Oracle: "When this creature enters, draw a card, then
/// discard a card."
pub fn prismari_cinderpoet() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cinderpoet",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Refactored in batch 40 to use the `etb_loot` shortcut.
        triggered_abilities: vec![crate::effect::shortcut::etb_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Pyrocaster — {3}{R}, 3/2 Human Wizard.
/// Synthesised Oracle: "When this creature enters, it deals 2 damage to
/// any target."
pub fn prismari_pyrocaster() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyrocaster",
        cost: cost(&[generic(3), r()]),
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

/// Prismari Drift — {U}{R}, Instant.
/// Synthesised Oracle: "This deals 2 damage to target creature. Scry 1."
pub fn prismari_drift() -> CardDefinition {
    CardDefinition {
        name: "Prismari Drift",
        cost: cost(&[u(), r()]),
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

/// Prismari Sparkbolt — {R}, Instant.
/// Synthesised Oracle: "This spell deals 2 damage to any target."
pub fn prismari_sparkbolt() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkbolt",
        cost: cost(&[crate::mana::r()]),
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

/// Prismari Stormrider — {3}{U}{R}, 3/3 Elemental Wizard with Flying.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, this creature gets +1/+0 until end of turn."
pub fn prismari_stormrider() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormrider",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
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

// ── Batch 39: 6 more Prismari cards ────────────────────────────────────────

/// Prismari Hothead — {1}{R}, 2/1 Human Wizard with Haste.
/// Synthesised Oracle: "Magecraft — Hothead gets +1/+0 EOT."
pub fn prismari_hothead() -> CardDefinition {
    CardDefinition {
        name: "Prismari Hothead",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Cantrip Bolt — {1}{U}{R}, Instant.
/// Synthesised Oracle: "Deal 2 damage to target creature and draw a card."
pub fn prismari_cantrip_bolt() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cantrip Bolt",
        cost: cost(&[generic(1), u(), r()]),
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

/// Prismari Wildmage — {2}{U}{R}, 3/2 Elemental Wizard.
/// Synthesised Oracle: "Magecraft — deal 1 damage to each opponent."
pub fn prismari_wildmage() -> CardDefinition {
    CardDefinition {
        name: "Prismari Wildmage",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
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

/// Prismari Stormbearer — {3}{U}{R}, 4/3 Elemental Wizard with Flying.
/// Synthesised Oracle: "ETB loot. Magecraft — Stormbearer gets +1/+0 EOT."
pub fn prismari_stormbearer() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormbearer",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Refactored in batch 40 to use the `etb_loot` shortcut.
        triggered_abilities: vec![
            crate::effect::shortcut::etb_loot(),
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

/// Prismari Pyromancer — {2}{R}, 2/3 Human Wizard.
/// Synthesised Oracle: "ETB deals 2 damage to any target."
pub fn prismari_pyromancer_v2() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyromancer V2",
        cost: cost(&[generic(2), r()]),
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

/// Prismari Tempestmage — {U}{R}, 2/2 Elemental Wizard.
/// Synthesised Oracle: "Magecraft — target creature gets +1/+0 EOT."
pub fn prismari_tempestmage() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Prismari Tempestmage",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
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

/// Prismari Cinderdrake — {4}{U}{R}, 4/4 Elemental Dragon with Flying.
/// Synthesised Oracle: "When this creature enters, deal 3 damage to any
/// target."
pub fn prismari_cinderdrake() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cinderdrake",
        cost: cost(&[generic(4), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
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

// ── Batch 40: more Prismari cards ───────────────────────────────────────────

/// Prismari Cinderbolt — {1}{U}{R}, 2/2 Human Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, this creature deals 1 damage to any target."
/// 3-mana magecraft ping body — the canonical Prismari aggro line.
pub fn prismari_cinderbolt() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cinderbolt",
        cost: cost(&[generic(1), u(), r()]),
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
        triggered_abilities: vec![crate::effect::shortcut::magecraft_ping_any(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Stormblade — {1}{R}, Instant.
/// Synthesised Oracle: "Prismari Stormblade deals 2 damage to any target.
/// Draw a card." 2-mana Bolt + cantrip — slow but reliable removal that
/// also fuels the spellslinger plan.
pub fn prismari_stormblade() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormblade",
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

/// Prismari Maestro — {2}{U}{R}, 2/4 Elemental Wizard.
/// Synthesised Oracle: "Whenever this creature deals combat damage to a
/// player, you may cast an instant or sorcery spell from your hand
/// without paying its mana cost." — approximated as plain combat-damage
/// "draw 2 cards" (no may-cast-free primitive on combat-damage).
pub fn prismari_maestro() -> CardDefinition {
    CardDefinition {
        name: "Prismari Maestro",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
            ),
            effect: Effect::Draw {
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

/// Prismari Emberscribe — {1}{R}, 2/1 Human Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, this creature deals 1 damage to target creature."
/// Aggressive 2-mana ping body — 2/1 frame is fragile but each
/// magecraft trigger pressures opp creatures.
pub fn prismari_emberscribe() -> CardDefinition {
    CardDefinition {
        name: "Prismari Emberscribe",
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Treasurer II — {2}{U}{R}, 2/3 Human Wizard.
/// Synthesised Oracle: "When this creature enters, create two Treasure
/// tokens." 4-mana double ramp.
pub fn prismari_treasurer_v2() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Treasurer II",
        cost: cost(&[generic(2), u(), r()]),
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
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: treasure_token(),
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

/// Prismari Quickcast — {U}{R} Instant.
/// Synthesised Oracle: "Deal 2 damage to any target. Draw a card."
/// 2-mana cantrip bolt — a strict-upgrade Shock at the cost of an
/// extra color requirement.
pub fn prismari_quickcast() -> CardDefinition {
    CardDefinition {
        name: "Prismari Quickcast",
        cost: cost(&[u(), r()]),
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

/// Prismari Starcaller — {3}{U}{R}, 3/3 Elemental Wizard Flying.
/// Synthesised Oracle: "Flying. When this creature enters, scry 2, then
/// draw a card." 5-mana value flier with selection + cantrip.
pub fn prismari_starcaller() -> CardDefinition {
    CardDefinition {
        name: "Prismari Starcaller",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
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

/// Prismari Scryer — {1}{U}{R}, 2/2 Elemental Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, scry 1." Pure top-deck-shaping body with each
/// cast.
pub fn prismari_scryer() -> CardDefinition {
    CardDefinition {
        name: "Prismari Scryer",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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


// ── Batch 42 (modern_decks) — Prismari expansion ────────────────────────────

/// Prismari Inferno II — {2}{R} Sorcery.
/// Synthesised Oracle: "Prismari Inferno deals 3 damage to any target."
/// 3-mana 3 damage Lava Spike-with-flexibility — Volcanic Hammer in
/// Prismari shells.
pub fn prismari_inferno_v2() -> CardDefinition {
    CardDefinition {
        name: "Prismari Inferno II",
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Glasshammer — {1}{R}, 2/2 Elemental Warrior.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, this creature deals 1 damage to each opponent." A
/// 2-mana spellslinger payoff in mono-red that doubles down on burn.
pub fn prismari_glasshammer() -> CardDefinition {
    CardDefinition {
        name: "Prismari Glasshammer",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Warrior],
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

/// Prismari Skywarp — {U}, Instant.
/// Synthesised Oracle: "Return target creature to its owner's hand."
/// 1-mana hard bounce — Unsummon flavour in Prismari colors.
pub fn prismari_skywarp() -> CardDefinition {
    CardDefinition {
        name: "Prismari Skywarp",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::OwnerOf(Box::new(
                Selector::Target(0),
            ))),
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

/// Prismari Stagewright — {2}{U}{R}, 3/3 Human Wizard.
/// Synthesised Oracle: "When this creature enters, draw a card. Magecraft
/// — Whenever you cast or copy an instant or sorcery spell, this creature
/// deals 1 damage to target creature or player." 4-mana value engine.
pub fn prismari_stagewright() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stagewright",
        cost: cost(&[generic(2), u(), r()]),
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
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            }),
            magecraft(Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Player),
                ),
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

/// Prismari Soundsmith — {U}{R}, 2/2 Elemental Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, this creature gets +1/+0 until end of turn." 2-mana
/// Prowess-shaped magecraft attacker — Monastery Swiftspear in Prismari
/// drag.
pub fn prismari_soundsmith() -> CardDefinition {
    CardDefinition {
        name: "Prismari Soundsmith",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
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


/// Prismari Pyroartist — {2}{R}, 2/3 Human Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, deal 1 damage to target creature or player." A
/// 3-mana magecraft ping body with a sturdier toughness than
/// Prismari Emberscribe.
pub fn prismari_pyroartist() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyroartist",
        cost: cost(&[generic(2), r()]),
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
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Player),
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

/// Prismari Brushpyre — {2}{U}{R}, 4/3 Elemental Wizard Haste.
/// Synthesised Oracle: "Haste. Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, this creature gets +1/+0 until end of turn."
/// 4-mana haste threat that becomes a magecraft snowballer.
pub fn prismari_brushpyre() -> CardDefinition {
    CardDefinition {
        name: "Prismari Brushpyre",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
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

// ── Batch 43 (modern_decks) — Prismari expansion ────────────────────────────

/// Prismari Blastcaster — {1}{R}, 2/1 Human Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, this creature deals 1 damage to target
/// creature." 2-mana magecraft removal-leaning ping body.
pub fn prismari_blastcaster() -> CardDefinition {
    CardDefinition {
        name: "Prismari Blastcaster",
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Oddsmaker — {U}{R}, 1/3 Human Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, scry 1." 2-mana magecraft selection body.
pub fn prismari_oddsmaker() -> CardDefinition {
    CardDefinition {
        name: "Prismari Oddsmaker",
        cost: cost(&[u(), r()]),
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

/// Prismari Glassforge — {2}{U}{R}, 2/3 Elemental Wizard Flying.
/// Synthesised Oracle: "Flying. When this creature enters, create a
/// Treasure token." 4-mana evasive ramp body.
pub fn prismari_glassforge() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Glassforge",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: treasure_token(),
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

/// Prismari Emberweaver — {3}{U}{R}, 4/3 Elemental Wizard Haste.
/// Synthesised Oracle: "Haste. When this creature enters, this
/// creature deals 2 damage to any target." 5-mana hasty ETB ping.
pub fn prismari_emberweaver() -> CardDefinition {
    CardDefinition {
        name: "Prismari Emberweaver",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::DealDamage {
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

/// Prismari Skyflare — {U}{R} Instant. Synthesised Oracle:
/// "This spell deals 2 damage to any target. Scry 1." 2-mana
/// instant burn + selection.
pub fn prismari_skyflare() -> CardDefinition {
    CardDefinition {
        name: "Prismari Skyflare",
        cost: cost(&[u(), r()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Volcanic Song — {3}{R} Sorcery. Synthesised Oracle:
/// "This spell deals 4 damage to target creature. Draw a card."
/// 4-mana headline burn + cantrip.
pub fn prismari_volcanic_song() -> CardDefinition {
    CardDefinition {
        name: "Prismari Volcanic Song",
        cost: cost(&[generic(3), r()]),
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

/// Prismari Inkjet Apprentice — {U}{R}, 2/2 Human Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, this creature deals 1 damage to each
/// opponent." 2-mana drain-burn engine on a body.
pub fn prismari_inkjet_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Prismari Inkjet Apprentice",
        cost: cost(&[u(), r()]),
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

// ── Batch 47 (modern_decks) — Prismari expansion ────────────────────────────

/// Prismari Scribbler — {1}{U}, 1/2 Human Wizard. Synthesised Oracle:
/// "When this creature enters, draw a card, then discard a card."
/// 2-mana loot enabler that feeds Prismari's discard-matter cards.
pub fn prismari_scribbler() -> CardDefinition {
    CardDefinition {
        name: "Prismari Scribbler",
        cost: cost(&[generic(1), u()]),
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
        triggered_abilities: vec![crate::effect::shortcut::etb_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Skyspark — {U}{R} Instant. Synthesised Oracle:
/// "Target creature gets +1/+1 and gains flying until end of turn.
/// Scry 1." 2-mana flash air-mail trick + smoothing.
pub fn prismari_skyspark() -> CardDefinition {
    CardDefinition {
        name: "Prismari Skyspark",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
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

/// Prismari Embershout — {2}{R} Sorcery. Synthesised Oracle:
/// "This spell deals 3 damage to target creature or player. Scry 1."
/// 3-mana flexible burn + smoothing.
pub fn prismari_embershout() -> CardDefinition {
    CardDefinition {
        name: "Prismari Embershout",
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

/// Prismari Stormcoil — {2}{U}{R}, 3/3 Elemental. Synthesised Oracle:
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// this creature gets +1/+1 until end of turn." Pump engine like
/// Colorstorm Stallion's small body, but flat 3/3 frame.
pub fn prismari_stormcoil() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormcoil",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Treasurespark — {1}{U}{R} Sorcery. Synthesised Oracle:
/// "Create a Treasure token. Draw a card." 3-mana ramp + draw.
pub fn prismari_treasurespark() -> CardDefinition {
    CardDefinition {
        name: "Prismari Treasurespark",
        cost: cost(&[generic(1), u(), r()]),
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
                definition: crate::game::effects::treasure_token(),
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

// ── Batch 48 (modern_decks) — Prismari expansion ────────────────────────────

/// Prismari Burnscribe — {1}{R}, 2/1 Human Wizard. Synthesised Oracle:
/// "When this creature enters, it deals 1 damage to target creature."
/// 2-mana ETB-ping body. Mirror of Lorehold Sparkflinger but at the
/// red-only slot.
pub fn prismari_burnscribe() -> CardDefinition {
    CardDefinition {
        name: "Prismari Burnscribe",
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
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

/// Prismari Treasurespell — {2}{U}{R} Instant. Synthesised Oracle:
/// "Create two Treasure tokens. Draw a card." 4-mana instant ramp +
/// cantrip. Galazeth Prismari engine fuel.
pub fn prismari_treasurespell() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Treasurespell",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: treasure_token(),
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

/// Prismari Sparkmage III — {U}{R}, 2/2 Human Wizard. Synthesised
/// Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, Prismari Sparkmage III deals 1 damage to target
/// creature." 2-mana magecraft creature-ping engine.
pub fn prismari_sparkmage_v3() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkmage III",
        cost: cost(&[u(), r()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Embergale — {3}{R} Sorcery. Synthesised Oracle: "Prismari
/// Embergale deals 3 damage to target creature. Prismari Embergale
/// deals 1 damage to each opponent." 4-mana headline burn + drain
/// tail. Same shape as Lorehold Ember-Forge in the mono-red slot.
pub fn prismari_embergale() -> CardDefinition {
    CardDefinition {
        name: "Prismari Embergale",
        cost: cost(&[generic(3), r()]),
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

/// Prismari Stormgale — {2}{U}{R}, 3/3 Elemental Wizard Flying.
/// Synthesised Oracle: "Flying. When this creature enters, draw a
/// card, then discard a card." 4-mana evasive looter top-end.
pub fn prismari_stormgale() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormgale",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 48 follow-up (modern_decks) — Prismari expansion 2 ────────────────

/// Prismari Flamewright — {2}{R}, 3/2 Human Wizard. Synthesised
/// Oracle: "When this creature enters, it deals 2 damage to any
/// target." 3-mana ETB-burn body.
pub fn prismari_flamewright() -> CardDefinition {
    CardDefinition {
        name: "Prismari Flamewright",
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

/// Prismari Cantrip Spark — {R} Instant. Synthesised Oracle: "Prismari
/// Cantrip Spark deals 1 damage to any target. Draw a card." 1-mana
/// cantrip-burn.
pub fn prismari_cantrip_spark() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cantrip Spark",
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

/// Prismari Dragonkin — {3}{U}{R}, 4/4 Drake Wizard Flying.
/// Synthesised Oracle: "Flying. When this creature enters, draw a
/// card." 5-mana evasive value body.
pub fn prismari_dragonkin() -> CardDefinition {
    CardDefinition {
        name: "Prismari Dragonkin",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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

/// Prismari Sparktwister — {U}{R}, 1/3 Elemental Wizard. Synthesised
/// Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, scry 1." 2-mana magecraft selection body.
pub fn prismari_sparktwister() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparktwister",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
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

/// Prismari Spelljay — {2}{R}{R} Sorcery. Synthesised Oracle:
/// "Prismari Spelljay deals 4 damage to target creature." 4-mana
/// big burn — Searing Spear / Magma Jet for creatures.
pub fn prismari_spelljay() -> CardDefinition {
    CardDefinition {
        name: "Prismari Spelljay",
        cost: cost(&[generic(2), r(), r()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 48 follow-up #2 (modern_decks) — more Prismari cards ──────────────

/// Prismari Quickburn — {R} Instant. Synthesised Oracle: "Prismari
/// Quickburn deals 2 damage to target creature." 1-mana Shock clone.
pub fn prismari_quickburn() -> CardDefinition {
    CardDefinition {
        name: "Prismari Quickburn",
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 49 (modern_decks) — more Prismari cards ───────────────────────────

/// Prismari Spellscribe — {U}{R}, 1/3 Human Wizard.
/// Synthesised Oracle: "Whenever you cast an instant or sorcery spell,
/// scry 1." 2-mana spellslinger filter body — Prismari's classic
/// 1/3 magecraft-scry anchor.
pub fn prismari_spellscribe() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Prismari Spellscribe",
        cost: cost(&[u(), r()]),
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

/// Prismari Sparkforge (v2) — {2}{R}, 3/2 Human Artificer.
/// Synthesised Oracle: "When this creature enters, create a Treasure
/// token." 3-mana value body — drops a 3/2 plus a ramp Treasure.
pub fn prismari_sparkforge_v2() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Sparkforge Anvil",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
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
                count: Value::Const(1),
                definition: treasure_token(),
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

/// Prismari Tidesinger — {1}{U}, 1/4 Merfolk Wizard.
/// Synthesised Oracle: "When this creature enters, return target
/// creature to its owner's hand." 2-mana ETB bounce — a Quandrix /
/// Prismari combo-trick that resets opp pressure.
pub fn prismari_tidesinger() -> CardDefinition {
    use crate::effect::PlayerRef::OwnerOf;
    CardDefinition {
        name: "Prismari Tidesinger",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
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
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(OwnerOf(Box::new(Selector::Target(0)))),
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

/// Prismari Searbolt — {1}{R} Instant. Synthesised Oracle:
/// "Prismari Searbolt deals 3 damage to target creature." Pure 2-mana
/// burn instant — Lightning Strike for Prismari decks.
pub fn prismari_searbolt() -> CardDefinition {
    CardDefinition {
        name: "Prismari Searbolt",
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Inkflame — {U}{R}, 2/2 Elemental Wizard. Synthesised
/// Oracle: "When this creature enters, draw a card, then discard a
/// card." 2-mana loot body.
pub fn prismari_inkflame() -> CardDefinition {
    CardDefinition {
        name: "Prismari Inkflame",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 50: Prismari synthesised cards ───────────────────────────────────

/// Prismari Bonfire — {1}{R}, Sorcery. Deals 3 damage to target
/// creature. 2-mana creature-only burn.
pub fn prismari_bonfire() -> CardDefinition {
    CardDefinition {
        name: "Prismari Bonfire",
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Snapcaster — {U}{R}, 2/1 Human Wizard. ETB Seq(Scry 1 +
/// Draw 1). 2-mana cantrip + smoothing.
pub fn prismari_snapcaster() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Prismari Snapcaster",
        cost: cost(&[u(), r()]),
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
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::Draw {
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

/// Prismari Pyrolancer — {2}{R}, 3/2 Human Wizard. Magecraft 1
/// damage to each opp. 3-mana drain-style magecraft.
pub fn prismari_pyrolancer() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyrolancer",
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

/// Prismari Drakemage — {3}{U}{R}, 3/3 Drake Wizard Flying. Magecraft
/// loot. 5-mana evasive looter.
pub fn prismari_drakemage() -> CardDefinition {
    use crate::effect::shortcut::magecraft_loot;
    CardDefinition {
        name: "Prismari Drakemage",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Cinder-Apprentice — {U}{R}, 1/2 Human Wizard. Magecraft
/// self-pump +1/+0 EOT. 2-mana magecraft prowess-like body.
pub fn prismari_cinder_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cinder-Apprentice",
        cost: cost(&[u(), r()]),
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

/// Prismari Pyroceptor — {2}{U}{R}, 3/3 Elemental Wizard. Magecraft
/// Seq(DealDamage 1 any + Scry 1). 4-mana ping + smoothing magecraft.
/// Disambiguated from the existing `prismari_pyrosage` factory earlier
/// in this file.
pub fn prismari_pyroceptor() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyroceptor",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
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

/// Prismari Coinforger — {1}{R}, 2/2 Human Wizard. ETB mints a Treasure
/// token. 2-mana ramp + body. Disambiguated from the existing
/// `prismari_tinkerer` factory in `extras`.
pub fn prismari_coinforger() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Coinforger",
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
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

/// Prismari Flashforge — {2}{R}, Instant. Seq(DealDamage 3 to
/// creature or player + Discard 1 + Draw 1). 3-mana burn + loot.
pub fn prismari_flashforge() -> CardDefinition {
    CardDefinition {
        name: "Prismari Flashforge",
        cost: cost(&[generic(2), r()]),
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
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
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

/// Prismari Riftspark — {U}{R}, 2/2 Elemental Wizard. Magecraft
/// MayDo(Discard + Draw). 2-mana optional loot magecraft.
pub fn prismari_riftspark() -> CardDefinition {
    CardDefinition {
        name: "Prismari Riftspark",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::MayDo {
            description: "Loot".to_string(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
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

/// Prismari Sparkwing — {3}{U}{R}, 3/3 Drake Wizard Flying + Haste.
/// 5-mana hasty evasive threat.
pub fn prismari_sparkwing() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkwing",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
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

/// Prismari Cantrip-Mage — {1}{U}, 1/2 Human Wizard. Magecraft
/// Scry 1 + Draw 1. Loot-on-cast magecraft body that smooths the
/// next IS cast.
pub fn prismari_cantrip_mage() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cantrip-Mage",
        cost: cost(&[generic(1), u()]),
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
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::Draw {
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

/// Prismari Firebrand — {1}{R}, 2/2 Human Wizard Haste. ETB
/// DealDamage 1 any target. 2-mana hasty ping body.
pub fn prismari_firebrand() -> CardDefinition {
    CardDefinition {
        name: "Prismari Firebrand",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
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

// ── batch 53: more Prismari cards ───────────────────────────────────────────

/// Prismari Emberveil — {2}{U}{R}, 3/2 Elemental Wizard. ETB Draw 1. 4-mana
/// cantrip body.
pub fn prismari_emberveil() -> CardDefinition {
    use crate::effect::shortcut::etb_draw;
    CardDefinition {
        name: "Prismari Emberveil",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_draw(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Firechord — {1}{R}, Instant. 3 damage to target creature.
/// Cheap creature-only burn.
pub fn prismari_firechord() -> CardDefinition {
    CardDefinition {
        name: "Prismari Firechord",
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Drakekin — {3}{U}{R}, 3/3 Drake Wizard Flying. ETB Scry 1.
/// 5-mana evasive scry body.
pub fn prismari_drakekin() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Prismari Drakekin",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
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

/// Prismari Inscribe — {R}, Sorcery. Seq(DealDamage 2 any + Scry 1).
/// Cheap burn-and-smooth.
pub fn prismari_inscribe() -> CardDefinition {
    CardDefinition {
        name: "Prismari Inscribe",
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

/// Prismari Pyremaster — {2}{R}, 3/3 Elemental Wizard. Magecraft 1 dmg
/// to any target. 3-mana magecraft burn body.
pub fn prismari_pyremaster() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Prismari Pyremaster",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── batch 54: more Prismari cards ───────────────────────────────────────────

/// Prismari Cinderpath — {2}{U}{R}, 3/3 Elemental Wizard. Magecraft
/// Seq(Draw 1 + Discard 1) — on-cast looter.
pub fn prismari_cinderpath() -> CardDefinition {
    use crate::effect::shortcut::magecraft_loot;
    CardDefinition {
        name: "Prismari Cinderpath",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Searstorm — {3}{R}, Sorcery. Deal 3 damage to target
/// creature + 2 damage to its controller.
pub fn prismari_searstorm() -> CardDefinition {
    CardDefinition {
        name: "Prismari Searstorm",
        cost: cost(&[generic(3), r()]),
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

/// Prismari Embertide — {1}{R}, 2/1 Elemental Haste. ETB DealDamage 1
/// to any target. Sparkmage-template aggressive haster.
pub fn prismari_embertide() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Prismari Embertide",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::DealDamage {
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

// ── Push (modern_decks, batch 55): 5 more Prismari cards ───────────────────

/// Prismari Stormcaller — {1}{U}{R}, 2/2 Elemental Wizard Prowess. Compact
/// Spectacle Mage-style aggressive prowess body at the {U}{R} slot.
pub fn prismari_stormcaller() -> CardDefinition {
    use crate::effect::shortcut::prowess;
    CardDefinition {
        name: "Prismari Stormcaller",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Prowess],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![prowess()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Embershock — {1}{R}, Instant. Deal 3 damage to target
/// creature. Compact Lightning Strike body for Prismari shells.
pub fn prismari_embershock() -> CardDefinition {
    CardDefinition {
        name: "Prismari Embershock",
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Spellscholar — {2}{U}, 1/3 Human Wizard. ETB Scry 2 +
/// magecraft Scry 1. Drawn out card selection on a defensive body.
pub fn prismari_spellscholar() -> CardDefinition {
    use crate::effect::shortcut::{etb, magecraft_scry};
    CardDefinition {
        name: "Prismari Spellscholar",
        cost: cost(&[generic(2), u()]),
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
        triggered_abilities: vec![
            etb(Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            }),
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

/// Prismari Reverberator — {3}{U}{R}, 3/3 Elemental Wizard. Magecraft
/// deal 2 damage to each opponent. Spell-slinger drain payoff.
pub fn prismari_reverberator() -> CardDefinition {
    CardDefinition {
        name: "Prismari Reverberator",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_each_opp(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Volcanist II — {3}{R}, Sorcery. Seq(DealDamage 4 target
/// creature + DealDamage 1 target player). 4-mana flexible split-damage
/// finisher.
pub fn prismari_volcanist_b55() -> CardDefinition {
    CardDefinition {
        name: "Prismari Volcanist II",
        cost: cost(&[generic(3), r()]),
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

// ── Push (modern_decks, batch 56) — new Prismari STX cards ─────────────────

/// Prismari Sparkleap — {U}{R}, 2/1 Elemental Haste + Prowess.
/// 2-mana aggressive prowess body with two relevant keywords for the
/// instant/sorcery shell.
pub fn prismari_sparkleap() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkleap",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste, Keyword::Prowess],
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

/// Prismari Flamewriter — {2}{U}{R}, 3/3 Elemental Wizard. Magecraft:
/// deal 1 damage to any target + Draw 1. Burn + draw scaling magecraft.
pub fn prismari_flamewriter() -> CardDefinition {
    CardDefinition {
        name: "Prismari Flamewriter",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
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
            Effect::Draw {
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

/// Prismari Cinderchant — {1}{R}, Instant. Deal 2 damage to any target
/// + Scry 1. 2-mana burn + selection.
pub fn prismari_cinderchant() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cinderchant",
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Skydrake — {3}{U}{R}, 3/3 Drake Wizard Flying + Prowess.
/// 5-mana evasive prowess finisher. Triggers on every cast.
pub fn prismari_skydrake() -> CardDefinition {
    CardDefinition {
        name: "Prismari Skydrake",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Prowess],
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

/// Prismari Floodfire — {3}{U}{R}, Sorcery. Deal 4 damage to target
/// player + Draw 2. 5-mana drain-and-draw finisher (analog of
/// Mind Spring + Burn).
pub fn prismari_floodfire() -> CardDefinition {
    CardDefinition {
        name: "Prismari Floodfire",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(4),
            },
            Effect::Draw {
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

// ── Push (modern_decks, batch 57): 3 more Prismari cards ───────────────────

/// Prismari Pyromage — {1}{R}, 2/2 Elemental Wizard with Haste.
/// Magecraft 1 damage to any target. 2-mana hasty magecraft pinger —
/// scales with every IS cast for sustained reach.
pub fn prismari_pyromage_b57() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Prismari Pyromage II",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
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

/// Prismari Stormcaller II — {1}{U}{R}, 2/2 Elemental Wizard with
/// Prowess. 3-mana prowess body — scales with every non-creature
/// spell.
pub fn prismari_stormcaller_v2() -> CardDefinition {
    use crate::effect::shortcut::prowess;
    CardDefinition {
        name: "Prismari Stormcaller II",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![prowess()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Sparkscribe — {1}{U}{R}, 2/2 Elemental Wizard with Flying.
/// ETB loot 1 (draw 1, discard 1). 3-mana evasive value engine.
pub fn prismari_sparkscribe_b57() -> CardDefinition {
    use crate::effect::shortcut::etb_loot;
    CardDefinition {
        name: "Prismari Sparkscribe II",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 58): 4 more Prismari cards ───────────────────

/// Prismari Apprentice II — {U}{R}, 2/2 Human Wizard. Magecraft: 1
/// damage to any target. Quintessential cheap Prismari ping body.
pub fn prismari_apprentice_b58() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Prismari Apprentice II",
        cost: cost(&[u(), r()]),
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
        activated_abilities: super::no_abilities(),
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

/// Prismari Flamewriter II — {2}{R}, 3/2 Elemental Wizard with Haste.
/// 3-mana hasty body — clean attacker for the Prismari aggressive
/// shell.
pub fn prismari_flamewriter_b58() -> CardDefinition {
    CardDefinition {
        name: "Prismari Flamewriter II",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
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

/// Prismari Tideflame — {1}{U}{R}, 2/3 Elemental Wizard. Magecraft: loot
/// (draw 1, discard 1). 3-mana selection body — Sparkscribe without
/// the flying, more efficient for the Magecraft chain.
pub fn prismari_tideflame() -> CardDefinition {
    use crate::effect::shortcut::magecraft_loot;
    CardDefinition {
        name: "Prismari Tideflame",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Stormcaster — {2}{U}{R}, 3/3 Elemental Wizard with Flying.
/// ETB Seq(1 damage any target + Scry 1). 4-mana evasive value body —
/// drops Bolt + selection on entry.
pub fn prismari_stormcaster_b58() -> CardDefinition {
    use crate::effect::shortcut::{etb, target_filtered};
    CardDefinition {
        name: "Prismari Stormcaster",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
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

// ── Push (modern_decks, batch 59): 5 more Prismari cards ───────────────────

/// Prismari Emberglyph — {U}{R}, 2/1 Human Wizard. Magecraft: each opp
/// loses 1 life. 2-mana cheap ping-each-opp body.
pub fn prismari_emberglyph() -> CardDefinition {
    CardDefinition {
        name: "Prismari Emberglyph",
        cost: cost(&[u(), r()]),
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
        activated_abilities: super::no_abilities(),
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

/// Prismari Iceforge — {1}{U}, 1/3 Merfolk Wizard. Magecraft: Scry 1.
/// 2-mana defensive smoother for spell-slinger shells.
pub fn prismari_iceforge() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Prismari Iceforge",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![magecraft_scry(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Flameseer — {2}{U}{R}, 3/2 Elemental Wizard with Haste.
/// Magecraft: loot (draw 1, discard 1). 4-mana hasty velocity body.
pub fn prismari_flameseer() -> CardDefinition {
    use crate::effect::shortcut::magecraft_loot;
    CardDefinition {
        name: "Prismari Flameseer",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Artificer — {3}{U}{R}, 3/4 Elemental Wizard. ETB Seq(create
/// a Treasure token + Scry 1). 5-mana mana-base + selection top-end.
pub fn prismari_artificer() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Prismari Artificer",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
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

// ── Push (modern_decks, batch 60): 3 more Prismari cards ───────────────────

/// Prismari Spell-Smith II — {U}{R}, 2/1 Human Wizard. Magecraft Scry 1.
/// 2-mana cheap selector — aggressive Prismari smoothing.
pub fn prismari_spell_smith_b60() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Prismari Spell-Smith II",
        cost: cost(&[u(), r()]),
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
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![magecraft_scry(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Fluxshaper — {2}{U}{R}, 3/3 Elemental Wizard with Flying.
/// Magecraft self-pump +1/+0 EOT. 4-mana evasive scaling body.
pub fn prismari_fluxshaper() -> CardDefinition {
    CardDefinition {
        name: "Prismari Fluxshaper",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
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

/// Prismari Glassblower — {3}{R}, 3/3 Elemental Wizard. ETB Seq(create
/// a Treasure token + DealDamage 1 any target). 4-mana ramp-and-ping.
pub fn prismari_glassblower() -> CardDefinition {
    use crate::effect::shortcut::{etb, target_filtered};
    CardDefinition {
        name: "Prismari Glassblower",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
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

/// Prismari Blast Apprentice — {1}{R}, 2/1 Human Wizard. Magecraft:
/// 1 damage to any target. 2-mana cheap ping body.
pub fn prismari_blast_apprentice() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Prismari Blast Apprentice",
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
        activated_abilities: super::no_abilities(),
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

// ── Push (modern_decks, batch 61): 5 more Prismari cards ────────────────────

/// Prismari Sparkscribe II — {U}{R}, 2/1 Human Wizard. Magecraft 1
/// damage any target via `magecraft_ping_any(1)`. 2-mana flexible ping
/// magecraft body (multicolor cousin of Prismari Blast Apprentice).
pub fn prismari_sparkscribe_b61() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Prismari Sparkscribe II",
        cost: cost(&[u(), r()]),
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
        activated_abilities: super::no_abilities(),
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

/// Prismari Emberforge — {2}{U}{R}, 2/3 Elemental Wizard. ETB Seq(mint
/// Treasure + ping 1 to target creature). 4-mana ramp-and-removal body.
pub fn prismari_emberforge() -> CardDefinition {
    use crate::effect::shortcut::etb;
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Emberforge",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
            },
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
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

/// Prismari Torchsmith — {3}{R}, 3/2 Elemental Wizard Haste. Magecraft
/// +1/+1 EOT self-pump. 4-mana aggressive haste magecraft body.
pub fn prismari_torchsmith() -> CardDefinition {
    CardDefinition {
        name: "Prismari Torchsmith",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
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

/// Prismari Iceshaper — {1}{U}{R}, 2/2 Elemental Wizard with Prowess.
/// 3-mana keyword-only body — Prowess scales the bear into a finisher
/// across a spell-heavy turn.
pub fn prismari_iceshaper() -> CardDefinition {
    CardDefinition {
        name: "Prismari Iceshaper",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Prowess],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
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

/// Prismari Smiteforge — {3}{U}{R}, 3/3 Elemental Wizard. ETB Seq(mint
/// Treasure + 2 damage to any target). 5-mana double-payoff value body.
pub fn prismari_smiteforge() -> CardDefinition {
    use crate::effect::shortcut::etb;
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Smiteforge",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
            },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
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

// ── Push (modern_decks, batch 62): 2 more Prismari cards ────────────────────

/// Prismari Sparksinger — {U}{R}, 2/2 Human Wizard. Magecraft ping each
/// opponent for 1 via `magecraft_ping_each_opp(1)`. 2-mana drain payoff
/// at the Prismari Apprentice slot.
pub fn prismari_sparksinger() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparksinger",
        cost: cost(&[u(), r()]),
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
        activated_abilities: super::no_abilities(),
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

/// Prismari Pyreforge — {2}{R}, 2/3 Elemental Wizard. ETB 1 damage any
/// target via the new `etb_ping_any(1)` shortcut. 3-mana cheap ping-on-
/// entry body — drops a Bolt for a 2/3 body at the curve.
pub fn prismari_pyreforge() -> CardDefinition {
    use crate::effect::shortcut::etb_ping_any;
    CardDefinition {
        name: "Prismari Pyreforge",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb_ping_any(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 63): 5 more Prismari cards ────────────────────

/// Prismari Goldcaster — {1}{R}, 2/2 Elemental Wizard. ETB mints a
/// Treasure token. 2-mana ramp body — pairs with Prismari spell-mana
/// payoffs and 5-mana Opus thresholds.
pub fn prismari_goldcaster() -> CardDefinition {
    use crate::effect::shortcut::etb;
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Goldcaster",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: treasure_token(),
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

/// Prismari Echoflame — {2}{U}{R}, Instant. Seq(DealDamage 2 + draw 1).
/// 4-mana burn + cantrip — Prismari's pet pattern at instant tempo.
pub fn prismari_echoflame() -> CardDefinition {
    use crate::card::SelectionRequirement;
    CardDefinition {
        name: "Prismari Echoflame",
        cost: cost(&[generic(2), u(), r()]),
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
            Effect::Draw {
                who: crate::card::Selector::You,
                amount: Value::Const(1),
            },
        ]),
        activated_abilities: super::no_abilities(),
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

/// Prismari Loresprite — {U}{R}, 2/1 Faerie Wizard Flying. Magecraft
/// Scry 1. 2-mana evasive smoothing body.
pub fn prismari_loresprite() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Prismari Loresprite",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![magecraft_scry(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Stormcaller II — {2}{U}{R}, 3/3 Elemental Wizard. ETB
/// Seq(mint Treasure + ping 1 to any target). 4-mana ramp + burn ETB.
pub fn prismari_stormcaller_b63() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::shortcut::etb;
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Stormcaller II",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: treasure_token(),
                count: Value::Const(1),
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

/// Prismari Combustomancer — {1}{R}, 2/2 Elemental Wizard. Magecraft 1
/// damage to any target. 2-mana magecraft burn engine — same shape as
/// Lorehold Sparkflinger but at the Prismari Wizard slot.
pub fn prismari_combustomancer() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Prismari Combustomancer",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: super::no_abilities(),
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

// ── Push (modern_decks, batch 65): 5 more Prismari cards ───────────────────

/// Prismari Sparkforger — {1}{U}{R}, 2/2 Elemental Wizard. ETB mint
/// 1 Treasure token. 3-mana ramp body for late-game spellslinger.
pub fn prismari_sparkforger() -> CardDefinition {
    use crate::effect::shortcut::etb;
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Sparkforger",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: treasure_token(),
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

/// Prismari Flashbinder — {U}{R}, 2/1 Elemental Wizard Prowess. 2-mana
/// aggressive Prowess body — counts every spell into the swing.
pub fn prismari_flashbinder() -> CardDefinition {
    CardDefinition {
        name: "Prismari Flashbinder",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Prowess],
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

/// Prismari Tidefurnace — {2}{U}{R}, Sorcery. Mints 1 Treasure token and
/// deals 2 damage to any target. 4-mana ramp + burn finisher.
pub fn prismari_tidefurnace() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Tidefurnace",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: treasure_token(),
                count: Value::Const(1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Embergloss — {1}{R}, 2/1 Elemental Wizard Haste. Magecraft
/// AddCounter +1/+1 on self. 2-mana hasty self-grower.
pub fn prismari_embergloss() -> CardDefinition {
    CardDefinition {
        name: "Prismari Embergloss",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
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

/// Prismari Stormtide — {1}{U}, 1/3 Merfolk Wizard Flying. Magecraft
/// loot 1 (Draw 1 + Discard 1). 2-mana evasive looter.
pub fn prismari_stormtide() -> CardDefinition {
    use crate::effect::shortcut::magecraft_loot;
    CardDefinition {
        name: "Prismari Stormtide",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 67): 6 more Prismari cards ───────────────────

/// Prismari Glassflame — {1}{R}, 2/1 Elemental Wizard. Magecraft pings
/// each opponent for 1. 2-mana red ping-each-opp magecraft body.
pub fn prismari_glassflame() -> CardDefinition {
    CardDefinition {
        name: "Prismari Glassflame",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Cinderdancer — {2}{R}, 3/2 Elemental Wizard Haste. Magecraft
/// self-pump +1/+0 EOT. 3-mana hasty Prowess-style aggressor.
pub fn prismari_cinderdancer() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cinderdancer",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Tidescryer — {2}{U}, 2/3 Merfolk Wizard. ETB Scry 2 via
/// `etb_scry(2)`. 3-mana defensive smoother body.
pub fn prismari_tidescryer() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Prismari Tidescryer",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_scry(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Magmaforge — {3}{U}{R}, Sorcery. Mints 2 Treasure tokens
/// and deals 3 damage to any target. 5-mana double-ramp + burn finisher.
pub fn prismari_magmaforge() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Magmaforge",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: treasure_token(),
                count: Value::Const(2),
            },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
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

/// Prismari Mistwarden — {U}{R}, 1/2 Elemental Wizard Flash. Magecraft
/// Scry 1 via `magecraft_scry(1)`. 2-mana flash blocker + selection.
pub fn prismari_mistwarden() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Prismari Mistwarden",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_scry(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Cinderspell — {R}, Instant. Deals 2 damage to any target.
/// Shock template at the {R} slot for Prismari's burn package.
pub fn prismari_cinderspell() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cinderspell",
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

// ── Push (modern_decks, batch 68): more Prismari U/R cards ───────────────

/// Prismari Sparkbearer — {U}{R}, 2/2 Elemental Wizard. ETB Mint Treasure
/// token. 2-mana ramp body that nets a Treasure (delayed mana).
pub fn prismari_sparkbearer() -> CardDefinition {
    use crate::effect::shortcut::etb;
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Sparkbearer",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: treasure_token(),
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

/// Prismari Stormcaller — {1}{R}, 2/1 Elemental Wizard Haste. Magecraft
/// 1 damage to any target. 2-mana hasty magecraft ping body.
pub fn prismari_stormcaller_b68() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Prismari Stormcaller (b68)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Flarewinder — {1}{U}, 1/3 Merfolk Wizard Flying. Magecraft
/// Scry 1. 2-mana defensive evasive Prismari smoother.
pub fn prismari_flarewinder() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Prismari Flarewinder",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_scry(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Brewbinder — {2}{U}{R}, 3/2 Elemental Wizard. ETB Mint
/// Treasure + Surveil 1. 4-mana ramp + selection body.
pub fn prismari_brewbinder() -> CardDefinition {
    use crate::effect::shortcut::etb;
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Brewbinder",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: treasure_token(),
                count: Value::Const(1),
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

/// Prismari Ember-Surge — {2}{U}{R}, Sorcery. Seq(DealDamage 3 + Draw 1).
/// 4-mana burn + cantrip.
pub fn prismari_ember_surge() -> CardDefinition {
    CardDefinition {
        name: "Prismari Ember-Surge",
        cost: cost(&[generic(2), u(), r()]),
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

// ── Batch 125 (push claude/modern_decks): four new Prismari cards ──────────

/// Prismari Blazewright (b125) — {2}{R}, 3/1 Human Wizard Haste.
/// Magecraft 1 damage to any target. 3-mana hasty magecraft burn body.
/// Uses the new `magecraft_ping_any` shortcut.
pub fn prismari_blazewright_b125() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Prismari Blazewright (b125)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Riftscholar (b125) — {1}{U}, 1/3 Human Wizard. ETB Seq(Scry
/// 1 + Draw 1). 2-mana selection + cantrip body.
pub fn prismari_riftscholar_b125() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Prismari Riftscholar (b125)",
        cost: cost(&[generic(1), u()]),
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
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::Draw {
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

/// Prismari Sparkshow (b125) — {U}{R}, Instant. Seq(DealDamage 2 to
/// any target + Draw 1). 2-mana burn + cantrip — Prismari's signature
/// "shock + draw" template.
pub fn prismari_sparkshow_b125() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkshow (b125)",
        cost: cost(&[u(), r()]),
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

/// Prismari Tempest-Bearer (b125) — {3}{U}{R}, 4/4 Elemental Wizard
/// Flying. ETB Seq(Draw 1 + Discard 1). 5-mana evasive top-end + loot
/// rider.
pub fn prismari_tempest_bearer_b125() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Prismari Tempest-Bearer (b125)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
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

// ── Batch 126 (push claude/modern_decks): five new Prismari cards ──────────

/// Prismari Cinderscholar (b126) — {1}{R}, 2/1 Human Wizard Haste.
/// Magecraft Loot (Draw 1 + Discard 1). 2-mana haste magecraft looter.
pub fn prismari_cinderscholar_b126() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cinderscholar (b126)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Riftrider (b126) — {U}{R}, 2/2 Human Pirate. Magecraft
/// self-pump +1/+0 EOT. Aggressive 2-mana magecraft scaler.
pub fn prismari_riftrider_b126() -> CardDefinition {
    CardDefinition {
        name: "Prismari Riftrider (b126)",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
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

/// Prismari Sparkstudent (b126) — {2}{U}{R}, 3/2 Human Wizard.
/// Magecraft Treasure mint via the new `magecraft_treasure` shortcut.
/// 4-mana ramp-on-cast engine — Treasures enable explosive turns.
pub fn prismari_sparkstudent_b126() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkstudent (b126)",
        cost: cost(&[generic(2), u(), r()]),
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
        triggered_abilities: vec![magecraft_treasure()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Tempest-Skipper (b126) — {3}{U}{R}, 3/3 Elemental Wizard
/// Flying. ETB Seq(Scry 2 + Draw 1). 5-mana evasive smoother + cantrip.
pub fn prismari_tempest_skipper_b126() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Prismari Tempest-Skipper (b126)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::Draw {
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

/// Prismari Coil-Caller (b126) — {U}{R} Instant. Seq(DealDamage 1 to
/// any target + Draw 1). 2-mana cheap shock-with-cantrip.
pub fn prismari_coil_caller_b126() -> CardDefinition {
    use crate::card::SelectionRequirement;
    CardDefinition {
        name: "Prismari Coil-Caller (b126)",
        cost: cost(&[u(), r()]),
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

// ── Batch 127 (push claude/modern_decks): new Prismari cards ──────────────

/// Prismari Sparkbolt (b127) — {1}{R} Instant. DealDamage 2 to a
/// target creature/player/PW + Draw 1. Prismari Shock-with-cantrip.
pub fn prismari_sparkbolt_b127() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkbolt (b127)",
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

/// Prismari Flarescholar (b127) — {2}{R}, 3/2 Human Wizard Haste.
/// Magecraft Treasure — 3-mana aggressive Treasure engine.
pub fn prismari_flarescholar_b127() -> CardDefinition {
    CardDefinition {
        name: "Prismari Flarescholar (b127)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Mistscholar (b127) — {1}{U}, 1/3 Human Wizard. Magecraft
/// Loot 1 — every spell triggers a Draw+Discard sequence.
pub fn prismari_mistscholar_b127() -> CardDefinition {
    CardDefinition {
        name: "Prismari Mistscholar (b127)",
        cost: cost(&[generic(1), u()]),
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
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Surgebearer (b127) — {3}{U}{R}, 4/3 Elemental Wizard.
/// Magecraft ping each opp 1. 5-mana race-breaker spellslinger.
pub fn prismari_surgebearer_b127() -> CardDefinition {
    CardDefinition {
        name: "Prismari Surgebearer (b127)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Ember-Wave (b127) — {U}{R} Instant. Tap target creature,
/// then DealDamage 1 to it. 2-mana tempo combat trick.
pub fn prismari_ember_wave_b127() -> CardDefinition {
    CardDefinition {
        name: "Prismari Ember-Wave (b127)",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
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

// ── Batch 128 (push claude/modern_decks): new Prismari cards ──────────────

/// Prismari Stormcrafter (b128) — {2}{U}{R}, 3/3 Elemental Wizard.
/// Magecraft loot — every instant/sorcery cycles the top card via
/// `magecraft_loot`. 4-mana spellslinger engine that pairs with
/// Treasure-mint Prismari payoffs.
pub fn prismari_stormcrafter_b128() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormcrafter (b128)",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Firebrand (b128) — {1}{R}, 2/1 Human Wizard with Haste.
/// Magecraft +1/+1 EOT self-pump — aggressive 2-drop that grows on
/// every spell. Same shape as Lorehold Cinderscholar but with haste
/// for surprise damage.
pub fn prismari_firebrand_b128() -> CardDefinition {
    CardDefinition {
        name: "Prismari Firebrand (b128)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
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

/// Prismari Tide-Surger (b128) — {3}{U}, 3/3 Merfolk Wizard Flying.
/// Magecraft Treasure mint — 4-mana flying treasure-engine for
/// Prismari ramp shells.
pub fn prismari_tide_surger_b128() -> CardDefinition {
    CardDefinition {
        name: "Prismari Tide-Surger (b128)",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Pyroblast (b128) — {1}{R} Instant. Deal 3 damage to any
/// target — Lightning Bolt at Prismari color identity. Bread-and-
/// butter burn for Prismari magecraft shells.
pub fn prismari_pyroblast_b128() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyroblast (b128)",
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

// ── Batch 129 (push claude/modern_decks): new Prismari cards ──────────────

/// Prismari Sparkmaker (b129) — {2}{U}{R}, 3/3 Elemental Wizard.
/// ETB Seq(mint Treasure + Scry 1). 4-mana fixing body that smooths
/// the deck while accelerating mana — pairs with Prismari Magma Opus
/// dreams.
pub fn prismari_sparkmaker_b129() -> CardDefinition {
    use crate::effect::shortcut::etb;
    use crate::effect::shortcut::mint_treasures;
    CardDefinition {
        name: "Prismari Sparkmaker (b129)",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            mint_treasures(1),
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

/// Prismari Tempestmage (b129) — {1}{U}{R}, 2/2 Human Wizard Prowess.
/// Magecraft draws a card. Aggressive 3-mana prowess body + magecraft
/// engine — chains spells into more cards.
pub fn prismari_tempestmage_b129() -> CardDefinition {
    use crate::effect::shortcut::magecraft_draw;
    CardDefinition {
        name: "Prismari Tempestmage (b129)",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Prowess],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_draw(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Inkwave (b129) — {1}{U} Instant. Counter target spell
/// unless its controller pays {2}. Standard Prismari soft counter at
/// 2 mana.
pub fn prismari_inkwave_b129() -> CardDefinition {
    CardDefinition {
        name: "Prismari Inkwave (b129)",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(2)]),
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

/// Prismari Stormbolt (b129) — {2}{R} Instant. Deal 4 damage to
/// target creature. Standard Prismari mid-cost burn at 3 mana,
/// upgrades to 4 from Pyroblast's 3.
pub fn prismari_stormbolt_b129() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormbolt (b129)",
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

// ── Batch 130 (push claude/modern_decks): more Prismari cards ───────────────

/// Prismari Emberseer (b130) — {1}{R}, 1/2 Human Wizard. Magecraft
/// creates a Treasure. Bottom-curve Prismari ramp creature that turns
/// every spell into a fuel token.
pub fn prismari_emberseer_b130() -> CardDefinition {
    CardDefinition {
        name: "Prismari Emberseer (b130)",
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
        triggered_abilities: vec![magecraft_treasure()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Inktrickster (b130) — {2}{U}{R}, 3/2 Human Wizard, Flying.
/// Magecraft Loot (draw + discard). An evasive Prismari with the
/// "smooth your draws" loop on every spell.
pub fn prismari_inktrickster_b130() -> CardDefinition {
    CardDefinition {
        name: "Prismari Inktrickster (b130)",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Burnstrike (b130) — {R}{R} Instant. Deal 4 damage to target
/// creature. Premium 2-mana hard burn — pairs with Treasure ramp from
/// Prismari Emberseer / Sparkmaker for explosive early kills.
pub fn prismari_burnstrike_b130() -> CardDefinition {
    CardDefinition {
        name: "Prismari Burnstrike (b130)",
        cost: cost(&[r(), r()]),
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

// ─── Batch 131: Prismari synthesised cards ───────────────────────────────────

/// Prismari Artistic Burst (b131) — {2}{U}{R} Sorcery. Seq(DealDamage 3
/// to any target + create Treasure token). 4-mana burn + Treasure-ramp.
pub fn prismari_artistic_burst_b131() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Artistic Burst (b131)",
        cost: cost(&[generic(2), u(), r()]),
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
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
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

/// Prismari Inkpyromancer (b131) — {1}{U}{R}, 2/2 Human Wizard.
/// Magecraft mints a Treasure token (`magecraft_treasure`).
pub fn prismari_inkpyromancer_b131() -> CardDefinition {
    CardDefinition {
        name: "Prismari Inkpyromancer (b131)",
        cost: cost(&[generic(1), u(), r()]),
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
        triggered_abilities: vec![magecraft_treasure()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Volatile Inkstroke (b131) — {U}{R} Instant. Seq(DealDamage
/// 2 to any target + Scry 1). 2-mana shock + smoothing.
pub fn prismari_volatile_inkstroke_b131() -> CardDefinition {
    CardDefinition {
        name: "Prismari Volatile Inkstroke (b131)",
        cost: cost(&[u(), r()]),
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

// ── Batch 132 ───────────────────────────────────────────────────────────────

/// Prismari Sparkscholar II (b132) — {U}{R}, 2/1 Human Wizard, Haste.
/// Magecraft: loot (draw 1, discard 1). Aggressive 2-drop spellslinger
/// with built-in card-quality engine.
pub fn prismari_sparkscholar_ii_b132() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkscholar II (b132)",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Glasswright (b132) — {2}{R}, 3/2 Human Artificer.
/// Magecraft: mint a Treasure. Treasure-engine body that scales with
/// instant/sorcery casts.
pub fn prismari_glasswright_b132() -> CardDefinition {
    CardDefinition {
        name: "Prismari Glasswright (b132)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Spellstrike (b132) — {2}{U}{R} Instant. Deal 3 damage to
/// any target, then draw a card. Big tempo instant.
pub fn prismari_spellstrike_b132() -> CardDefinition {
    CardDefinition {
        name: "Prismari Spellstrike (b132)",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                amount: Value::Const(3),
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
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

/// Prismari Tempest-Scribe (b132) — {3}{U}, 2/4 Human Wizard. Flying.
/// Magecraft self-pump +1/+0 EOT. Defensive flier that turns into a
/// 3/4 attacker with each instant/sorcery cast.
pub fn prismari_tempest_scribe_b132() -> CardDefinition {
    CardDefinition {
        name: "Prismari Tempest-Scribe (b132)",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
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

// ── Batch 133 ───────────────────────────────────────────────────────────────

/// Prismari Ember-Sprite (b133) — {1}{R}, 2/1 Elemental, Haste.
/// Magecraft: deal 1 damage to any target.
pub fn prismari_ember_sprite_b133() -> CardDefinition {
    CardDefinition {
        name: "Prismari Ember-Sprite (b133)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            amount: Value::Const(1),
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
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

/// Prismari Wave-Surger (b133) — {2}{U}, 2/3 Merfolk Wizard.
/// ETB Scry 1, then Draw 1. Uses the new `etb_scry_and_draw` shortcut.
pub fn prismari_wave_surger_b133() -> CardDefinition {
    use crate::effect::shortcut::etb_scry_and_draw;
    CardDefinition {
        name: "Prismari Wave-Surger (b133)",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_scry_and_draw(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Magma-Cleric (b133) — {2}{R}, 3/3 Human Wizard.
/// Vanilla beat-stick with the Wizard tribal subtype for Wizard-tribal
/// pools.
pub fn prismari_magma_cleric_b133() -> CardDefinition {
    CardDefinition {
        name: "Prismari Magma-Cleric (b133)",
        cost: cost(&[generic(2), r()]),
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

// ── Batch 135 ───────────────────────────────────────────────────────────────

/// Prismari Sparkmage (b135) — {1}{U}{R} 2/3 Human Wizard.
/// Magecraft: deal 1 damage to any target. The canonical Prismari
/// "every-spell-pings" engine on a defensive body.
pub fn prismari_sparkmage_b135() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Prismari Sparkmage (b135)",
        cost: cost(&[generic(1), u(), r()]),
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

/// Prismari Splash (b135) — {U}{R} Instant. Draw a card, then deal 1
/// damage to target creature or player. Cheap cantrip + ping — fuels
/// magecraft triggers and replaces itself.
pub fn prismari_splash_b135() -> CardDefinition {
    CardDefinition {
        name: "Prismari Splash (b135)",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
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

/// Prismari Glasswright II (b135) — {2}{R} 2/3 Human Artificer.
/// Magecraft: create a Treasure token. Wider variant of the b132
/// Prismari Glasswright (which had P/T 3/2) — flips a defensive body
/// and a Treasure engine.
pub fn prismari_glasswright_ii_b135() -> CardDefinition {
    CardDefinition {
        name: "Prismari Glasswright II (b135)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Stormcaller (b135) — {3}{U}{R} 3/3 Elemental, Haste.
/// Magecraft: target creature an opponent controls gets -1/-1 EOT.
/// Aggressive Spellsplit body — every spell shrinks blockers.
pub fn prismari_stormcaller_b135() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Prismari Stormcaller (b135)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
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

// ── Batch 136 ───────────────────────────────────────────────────────────────

/// Prismari Ember-Scribe (b136) — {2}{U}{R} 3/3 Human Wizard.
/// Magecraft Seq(DealDamage 1 any + Draw 1). Spellsling engine — both
/// pings and draws on every spell.
pub fn prismari_ember_scribe_b136() -> CardDefinition {
    CardDefinition {
        name: "Prismari Ember-Scribe (b136)",
        cost: cost(&[generic(2), u(), r()]),
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
            Effect::Draw {
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

/// Prismari Burnpaste (b136) — {1}{R} Instant. Deal 3 damage to target
/// creature. Cheap red removal at the 2-mana slot.
pub fn prismari_burnpaste_b136() -> CardDefinition {
    CardDefinition {
        name: "Prismari Burnpaste (b136)",
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Treasure-Pyro (b136) — {3}{R} 4/2 Human Artificer. ETB
/// creates a Treasure token. Heavy hitter with built-in ramp.
pub fn prismari_treasure_pyro_b136() -> CardDefinition {
    use crate::card::TriggeredAbility;
    use crate::effect::{EventScope, EventSpec};
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Treasure-Pyro (b136)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
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

// ── Batch 138 ───────────────────────────────────────────────────────────────

/// Prismari Sparkforge (b138) — {1}{U}{R} 2/3 Human Artificer. ETB
/// mints a Treasure token. 3-mana ramp body for spell-heavy shells.
pub fn prismari_sparkforge_b138() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Sparkforge (b138)",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
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
                definition: treasure_token(),
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

/// Prismari Embersinger (b138) — {1}{R} 2/2 Elemental Bard.
/// Magecraft 1 damage to each opp. 2-mana spellslinger that drains
/// over time.
pub fn prismari_embersinger_b138() -> CardDefinition {
    CardDefinition {
        name: "Prismari Embersinger (b138)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Bard],
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

/// Prismari Surgebolt (b138) — {1}{R} Instant. DealDamage 3 to any
/// target. Cheap 2-mana burn spell.
pub fn prismari_surgebolt_b138() -> CardDefinition {
    CardDefinition {
        name: "Prismari Surgebolt (b138)",
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

/// Prismari Wavecaller (b138) — {1}{U} 1/3 Merfolk Wizard. Magecraft
/// Loot (draw, discard). Cheap card-selection body.
pub fn prismari_wavecaller_b138() -> CardDefinition {
    CardDefinition {
        name: "Prismari Wavecaller (b138)",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Stormhand (b138) — {2}{U}{R} Sorcery.
/// Seq(DealDamage 3 to target + Treasure mint). 4-mana burn + ramp.
pub fn prismari_stormhand_b138() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Stormhand (b138)",
        cost: cost(&[generic(2), u(), r()]),
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
                definition: treasure_token(),
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

// ── Batch 139 ───────────────────────────────────────────────────────────────

/// Prismari Flarewright (b139) — {1}{U}{R} 3/2 Elemental Bard.
/// Magecraft +1/+1 EOT self-pump (magecraft_self_pump shape).
pub fn prismari_flarewright_b139() -> CardDefinition {
    CardDefinition {
        name: "Prismari Flarewright (b139)",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
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

/// Prismari Shocksinger (b139) — {1}{R} Sorcery. Seq(DealDamage 2
/// + Treasure mint). 2-mana burn + ramp.
pub fn prismari_shocksinger_b139() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Shocksinger (b139)",
        cost: cost(&[generic(1), r()]),
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
                definition: treasure_token(),
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

/// Prismari Glassflinger (b136) — {U}{R} 2/2 Human Wizard. Magecraft
/// scry 1. Cheap evasion-less Wizard with smoothing on every spell.
pub fn prismari_glassflinger_b136() -> CardDefinition {
    CardDefinition {
        name: "Prismari Glassflinger (b136)",
        cost: cost(&[u(), r()]),
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

// ── Batch 141 ───────────────────────────────────────────────────────────────

/// Prismari Magma-Channeler (b141) — {1}{U}{R} 2/3 Human Wizard.
/// Magecraft Treasure mint. Standard treasure-on-cast Prismari engine.
pub fn prismari_magma_channeler_b141() -> CardDefinition {
    CardDefinition {
        name: "Prismari Magma-Channeler (b141)",
        cost: cost(&[generic(1), u(), r()]),
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
        triggered_abilities: vec![magecraft_treasure()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Pyromage (b141) — {1}{R} 2/2 Elemental Wizard.
/// Magecraft ping-any 1. Spellslinger ping engine.
pub fn prismari_pyromage_b141() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Prismari Pyromage (b141)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Tidalstorm (b141) — {U}{R} Instant.
/// Seq(DealDamage 2 + Draw 1). 2-mana burn cantrip.
pub fn prismari_tidalstorm_b141() -> CardDefinition {
    CardDefinition {
        name: "Prismari Tidalstorm (b141)",
        cost: cost(&[u(), r()]),
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

/// Prismari Embergeist (b141) — {2}{U}{R} 3/3 Spirit Elemental Flying.
/// Magecraft Loot. Spellslinger evasive flyer with selection on every
/// spell.
pub fn prismari_embergeist_b141() -> CardDefinition {
    CardDefinition {
        name: "Prismari Embergeist (b141)",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
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

/// Prismari Surgemage (b142) — {1}{U}{R} 2/2 Human Wizard. Magecraft
/// draw a card. Spellslinger card-engine (Archmage Emeritus cousin
/// at a smaller body).
pub fn prismari_surgemage_b142() -> CardDefinition {
    use crate::effect::shortcut::magecraft_draw;
    CardDefinition {
        name: "Prismari Surgemage (b142)",
        cost: cost(&[generic(1), u(), r()]),
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
        triggered_abilities: vec![magecraft_draw(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Cinderwave (b142) — {2}{U}{R} Instant. Deal 3 damage to
/// any target + Draw 1. 4-mana burn cantrip.
pub fn prismari_cinderwave_b142() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cinderwave (b142)",
        cost: cost(&[generic(2), u(), r()]),
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

/// Prismari Tidemaster (b142) — {3}{U}{R} 3/4 Elemental Wizard Flying.
/// ETB create a Treasure token. 5-mana ramp + evasive body.
pub fn prismari_tidemaster_b142() -> CardDefinition {
    use crate::effect::shortcut::{etb, mint_treasures};
    CardDefinition {
        name: "Prismari Tidemaster (b142)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(mint_treasures(1))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Pyrocaster (b142) — {1}{R} 2/1 Human Wizard. ETB Loot
/// (draw + discard). Tempo body for cards-in-graveyard payoffs.
pub fn prismari_pyrocaster_b142() -> CardDefinition {
    use crate::effect::shortcut::etb_loot;
    CardDefinition {
        name: "Prismari Pyrocaster (b142)",
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
        triggered_abilities: vec![etb_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Magmarush (b142) — {3}{R} Sorcery. Deal 5 damage to
/// target creature. Mid-game hard creature removal.
pub fn prismari_magmarush_b142() -> CardDefinition {
    CardDefinition {
        name: "Prismari Magmarush (b142)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 143 ───────────────────────────────────────────────────────────────

/// Prismari Pyroartist (b143) — {U}{R} 2/2 Human Wizard. Magecraft
/// each opp loses 1 life.
pub fn prismari_pyroartist_b143() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyroartist (b143)",
        cost: cost(&[u(), r()]),
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

/// Prismari Cantripflinger (b143) — {1}{U} Instant. Deal 2 damage to
/// target creature and draw a card. Cheap-cantrip burn.
pub fn prismari_cantripflinger_b143() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cantripflinger (b143)",
        cost: cost(&[generic(1), u()]),
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

/// Prismari Stormcaster (b143) — {2}{U}{R} 3/3 Elemental Wizard.
/// Magecraft create a Treasure token + magecraft self-pump +1/+0 EOT.
/// (Splits into two triggers.)
pub fn prismari_stormcaster_b143() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormcaster (b143)",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_treasure(), magecraft_self_pump(1, 0)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Cantriplord (b143) — {2}{R} Sorcery. Deal 3 damage to any
/// target and draw 2 cards. 3-mana burn + heavy draw finisher.
pub fn prismari_cantriplord_b143() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cantriplord (b143)",
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
            Effect::Draw {
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

/// Prismari Elementalmage (b143) — {3}{U}{R} 4/4 Elemental Wizard.
/// 5-mana vanilla curve-topper.
pub fn prismari_elementalmage_b143() -> CardDefinition {
    CardDefinition {
        name: "Prismari Elementalmage (b143)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 144 ───────────────────────────────────────────────────────────────

/// Prismari Stormgust (b144) — {U}{R} Instant. Deal 2 damage to target
/// creature and draw a card. 2-mana removal + cantrip.
pub fn prismari_stormgust_b144() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormgust (b144)",
        cost: cost(&[u(), r()]),
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
            Effect::Draw {
                who: crate::effect::Selector::You,
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

/// Prismari Ember-Cantor (b144) — {1}{R} 2/2 Human Wizard. Cycling {1}{U}.
pub fn prismari_ember_cantor_b144() -> CardDefinition {
    CardDefinition {
        name: "Prismari Ember-Cantor (b144)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Cycling(cost(&[generic(1), u()]))],
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

// ── Batch 145 ───────────────────────────────────────────────────────────────

/// Prismari Frosthand (b145) — {1}{U} 1/3 Human Wizard. ETB tap target
/// opp creature. Defensive tempo body.
pub fn prismari_frosthand_b145() -> CardDefinition {
    use crate::effect::shortcut::etb_tap_opp_creature;
    CardDefinition {
        name: "Prismari Frosthand (b145)",
        cost: cost(&[generic(1), u()]),
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
        triggered_abilities: vec![etb_tap_opp_creature()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Magmasplitter (b145) — {2}{R}{R} Sorcery. Deal 4 damage to
/// target creature. 4-mana burn finisher.
pub fn prismari_magmasplitter_b145() -> CardDefinition {
    CardDefinition {
        name: "Prismari Magmasplitter (b145)",
        cost: cost(&[generic(2), r(), r()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Wavecraft (b144) — {3}{U}{R} 4/3 Elemental Wizard Flying.
/// Magecraft loot. 5-mana race-breaker + selection.
pub fn prismari_wavecraft_b144() -> CardDefinition {
    CardDefinition {
        name: "Prismari Wavecraft (b144)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Volcanist (b143) — {1}{R} 2/2 Human Wizard. ETB deal 2 damage
/// to target creature. Compact tempo body.
pub fn prismari_volcanist_b143() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Prismari Volcanist (b143)",
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
        triggered_abilities: vec![etb(Effect::DealDamage {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 146 ───────────────────────────────────────────────────────────────

/// Prismari Pyromage (b146) — {2}{R} 3/2 Human Wizard. Magecraft 1
/// damage to any target. Repeatable ping in a spellslinger shell.
pub fn prismari_pyromage_b146() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Prismari Pyromage (b146)",
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

/// Prismari Volcanic Spell (b146) — {2}{R} Sorcery. Deal 3 damage to any
/// target. Lava Spike adjacent at +1 mana for "any target" reach.
pub fn prismari_volcanic_spell_b146() -> CardDefinition {
    CardDefinition {
        name: "Prismari Volcanic Spell (b146)",
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Sleetcaster (b146) — {1}{U} 1/3 Human Wizard. ETB tap target
/// opp creature. 2-mana tempo body.
pub fn prismari_sleetcaster_b146() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Prismari Sleetcaster (b146)",
        cost: cost(&[generic(1), u()]),
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Treasurer (b146) — {2}{U}{R} 2/3 Human Wizard. Magecraft
/// create a Treasure token. Treasure-engine body — Storm-Kiln Artist
/// at a different body.
pub fn prismari_treasurer_b146() -> CardDefinition {
    CardDefinition {
        name: "Prismari Treasurer (b146)",
        cost: cost(&[generic(2), u(), r()]),
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
        triggered_abilities: vec![magecraft_treasure()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Mage-Champion (b146) — {3}{U}{R} 4/3 Elemental Wizard Haste.
/// 5-mana hasty finisher. Pairs with Prismari Pledgemage's trample.
pub fn prismari_mage_champion_b146() -> CardDefinition {
    CardDefinition {
        name: "Prismari Mage-Champion (b146)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Charge (b146) — {U}{R} Instant. Seq(Draw 1 + DealDamage 1
/// to any target). 2-mana cantrip-burn — slightly better Spectral
/// Strike at color identity tradeoff.
pub fn prismari_charge_b146() -> CardDefinition {
    CardDefinition {
        name: "Prismari Charge (b146)",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
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

/// Prismari Reflectionist (b146) — {2}{U} 2/3 Human Wizard. ETB Scry 2.
/// 3-mana dig body for spell-heavy shells.
pub fn prismari_reflectionist_b146() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Prismari Reflectionist (b146)",
        cost: cost(&[generic(2), u()]),
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
        triggered_abilities: vec![etb_scry(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Pyrolancer (b146) — {1}{R} 2/2 Human Wizard. Magecraft
/// +1/+1 counter on this creature. Self-growing 2-drop magecraft.
pub fn prismari_pyrolancer_b146() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyrolancer (b146)",
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

/// Prismari Tidemage (b146) — {1}{U} 1/2 Human Wizard. ETB return target
/// creature to its owner's hand. 2-mana Unsummon-on-a-body — bounce
/// tempo with a chump-blocker rider.
pub fn prismari_tidemage_b146() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Prismari Tidemage (b146)",
        cost: cost(&[generic(1), u()]),
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
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
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

/// Prismari Surge (b146) — {3}{U}{R} Sorcery. Seq(DealDamage 4 to any
/// target + Draw 1). 5-mana burst-burn + cantrip.
pub fn prismari_surge_b146() -> CardDefinition {
    CardDefinition {
        name: "Prismari Surge (b146)",
        cost: cost(&[generic(3), u(), r()]),
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
                amount: Value::Const(4),
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

// ── Batch 147 ───────────────────────────────────────────────────────────────

/// Prismari Embercaller (b147) — {2}{R} 3/3 Human Wizard. Magecraft 1
/// damage to each opp. Drain-burn template.
pub fn prismari_embercaller_b147() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_each_opp;
    CardDefinition {
        name: "Prismari Embercaller (b147)",
        cost: cost(&[generic(2), r()]),
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

/// Prismari Tidescribe (b147) — {1}{U} 1/2 Human Wizard. Magecraft loot.
/// Same shape as Quandrix Mathwitch at the {1}{U} slot.
pub fn prismari_tidescribe_b147() -> CardDefinition {
    CardDefinition {
        name: "Prismari Tidescribe (b147)",
        cost: cost(&[generic(1), u()]),
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
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Flamekind (b147) — {2}{R}{R} 4/3 Elemental Haste Trample.
/// 4-mana hasty trampler — Prismari's beefy finisher.
pub fn prismari_flamekind_b147() -> CardDefinition {
    CardDefinition {
        name: "Prismari Flamekind (b147)",
        cost: cost(&[generic(2), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Counterscribe (b147) — {1}{U} Instant. Counter target spell
/// unless its controller pays {1}. Soft counter — Spell Pierce template
/// at a different color/cost combo.
pub fn prismari_counterscribe_b147() -> CardDefinition {
    CardDefinition {
        name: "Prismari Counterscribe (b147)",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: Selector::Target(0),
            mana_cost: cost(&[generic(1)]),
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

/// Prismari Arcanist (b147) — {3}{U}{R} 3/3 Human Wizard Flying.
/// Magecraft Scry 1 + Draw 1. 5-mana premium spellslinger value flier.
pub fn prismari_arcanist_b147() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry_and_draw;
    CardDefinition {
        name: "Prismari Arcanist (b147)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_scry_and_draw(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 148 ───────────────────────────────────────────────────────────────

/// Prismari Sparkmage (b148) — {1}{R} 2/1 Human Wizard Haste. Magecraft
/// 1 damage to any target. 2-mana hasty magecraft ping engine.
pub fn prismari_sparkmage_b148() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Prismari Sparkmage (b148)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Splashmage (b148) — {U}{R} Instant. Seq(DealDamage 1 + Draw 1).
/// 2-mana cantrip ping — see Prismari Charge for the bigger version.
pub fn prismari_splashmage_b148() -> CardDefinition {
    CardDefinition {
        name: "Prismari Splashmage (b148)",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(1),
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

/// Prismari Treasurehunter (b148) — {2}{U}{R} 3/3 Human Wizard. ETB
/// mint 1 Treasure + Scry 1.
pub fn prismari_treasurehunter_b148() -> CardDefinition {
    use crate::effect::shortcut::mint_treasures;
    CardDefinition {
        name: "Prismari Treasurehunter (b148)",
        cost: cost(&[generic(2), u(), r()]),
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
        triggered_abilities: vec![{
            use crate::effect::shortcut::etb;
            etb(Effect::Seq(vec![
                mint_treasures(1),
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
            ]))
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

/// Prismari Brawler (b148) — {2}{R} 3/2 Human Warrior Haste. Vanilla
/// aggressive 3-drop — Prismari's red weight.
pub fn prismari_brawler_b148() -> CardDefinition {
    CardDefinition {
        name: "Prismari Brawler (b148)",
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Mindstrike (b148) — {2}{U}{R} Sorcery. DealDamage 4 to
/// target creature + draw 1. 4-mana finisher + cantrip.
pub fn prismari_mindstrike_b148() -> CardDefinition {
    CardDefinition {
        name: "Prismari Mindstrike (b148)",
        cost: cost(&[generic(2), u(), r()]),
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

// ── Batch 149 ───────────────────────────────────────────────────────────────

/// Prismari Etherealist (b149) — {1}{U}{R} 2/2 Human Wizard Flying +
/// Haste. Aggressive evasive 3-drop.
pub fn prismari_etherealist_b149() -> CardDefinition {
    CardDefinition {
        name: "Prismari Etherealist (b149)",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Stormbringer (b149) — {3}{U}{R} 3/4 Elemental Trample +
/// Haste. Top-curve aggressor.
pub fn prismari_stormbringer_b149() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormbringer (b149)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Haste],
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

// ── Batch 150 ───────────────────────────────────────────────────────────────

/// Prismari Pyromage (b150) — {1}{R} 2/2 Elemental Wizard. Magecraft
/// 2 damage to each opponent — burn engine.
pub fn prismari_pyromage_b150() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyromage (b150)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_each_opp(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Tidemage (b150) — {2}{U} 1/3 Merfolk Wizard. Magecraft
/// scry 2 — card selection engine.
pub fn prismari_tidemage_b150() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Prismari Tidemage (b150)",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_scry(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Galemaster (b150) — {2}{U}{R} 3/3 Elemental Wizard.
/// Flying + Haste — strong evasive 4-drop.
pub fn prismari_galemaster_b150() -> CardDefinition {
    CardDefinition {
        name: "Prismari Galemaster (b150)",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
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

/// Prismari Stormcaller (b150) — {1}{U}{R} 2/3 Elemental Wizard.
/// Magecraft loot (draw 1, discard 1) — recurring rummager.
pub fn prismari_stormcaller_b150() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormcaller (b150)",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Treasure-Smith (b150) — {3}{U}{R} 3/4 Elemental Wizard.
/// Magecraft mint a Treasure token.
pub fn prismari_treasure_smith_b150() -> CardDefinition {
    CardDefinition {
        name: "Prismari Treasure-Smith (b150)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Inferno (b150) — {3}{R} Sorcery. Deal 3 damage divided
/// (approximated to one target — Crackle pattern). Standard Prismari
/// burn-pile spell.
pub fn prismari_inferno_b150() -> CardDefinition {
    CardDefinition {
        name: "Prismari Inferno (b150)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Player),
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

/// Prismari Aetherwave (b150) — {U}{R} Instant. Draw 2, discard 1.
/// Cheap card selection / filter.
pub fn prismari_aetherwave_b150() -> CardDefinition {
    CardDefinition {
        name: "Prismari Aetherwave (b150)",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
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

// ── Batch 151 ───────────────────────────────────────────────────────────────

/// Prismari Brawler (b151) — {2}{R} 3/3 Elemental Warrior Haste.
/// Aggressive 3-mana hasty body.
pub fn prismari_brawler_b151() -> CardDefinition {
    CardDefinition {
        name: "Prismari Brawler (b151)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Warrior],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Inferno-Tide (b151) — {3}{U}{R} Sorcery. Deal 2 damage to
/// each opponent + draw 2 cards. Two-axis spell.
pub fn prismari_inferno_tide_b151() -> CardDefinition {
    CardDefinition {
        name: "Prismari Inferno-Tide (b151)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            Effect::Draw {
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

/// Prismari Glassblower (b151) — {2}{U} 2/3 Elemental Artificer.
/// Magecraft mint a Treasure token. Combos with mana sinks.
pub fn prismari_glassblower_b151() -> CardDefinition {
    CardDefinition {
        name: "Prismari Glassblower (b151)",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Wavecaller (b151) — {1}{U}{R} 2/2 Elemental Wizard.
/// Magecraft scry 1.
pub fn prismari_wavecaller_b151() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Prismari Wavecaller (b151)",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Stormcrest (b151) — {3}{R} 3/2 Elemental Dragon Flying +
/// Haste. Big finisher with both evasion and haste.
pub fn prismari_stormcrest_b151() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormcrest (b151)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Dragon],
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

// ── Batch 153 ───────────────────────────────────────────────────────────────

/// Prismari Spellburst (b153) — {1}{U}{R} Instant. Counter target spell
/// unless its controller pays {3}.
pub fn prismari_spellburst_b153() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::mana::cost as mc;
    use crate::mana::generic as gc;
    CardDefinition {
        name: "Prismari Spellburst (b153)",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: mc(&[gc(3)]),
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

/// Prismari Elementalist (b153) — {2}{U}{R} 3/3 Elemental Wizard.
/// Magecraft mint Treasure + scry 1.
pub fn prismari_elementalist_b153() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Prismari Elementalist (b153)",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Spellsplash (b153) — {2}{R} Sorcery. Deal 4 damage to
/// target creature.
pub fn prismari_spellsplash_b153() -> CardDefinition {
    CardDefinition {
        name: "Prismari Spellsplash (b153)",
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 154 ───────────────────────────────────────────────────────────────

/// Prismari Treasurelord (b154) — {2}{U}{R} 3/3 Human Wizard.
/// Magecraft Treasure mint via `magecraft_treasure()`. Compact ramp +
/// engine top-end.
pub fn prismari_treasurelord_b154() -> CardDefinition {
    CardDefinition {
        name: "Prismari Treasurelord (b154)",
        cost: cost(&[generic(2), u(), r()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Inferno (b154) — {3}{R} Instant. Deal 5 damage to target
/// creature or planeswalker. 4-mana hard burn.
pub fn prismari_inferno_b154() -> CardDefinition {
    CardDefinition {
        name: "Prismari Inferno (b154)",
        cost: cost(&[generic(3), r()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Tempestmage (b154) — {1}{U} 1/2 Human Wizard.
/// Magecraft self-pump +1/+1 EOT — small-body scaling magecraft.
pub fn prismari_tempestmage_b154() -> CardDefinition {
    CardDefinition {
        name: "Prismari Tempestmage (b154)",
        cost: cost(&[generic(1), u()]),
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

/// Prismari Crashbinder (b154) — {3}{U}{R} 4/4 Elemental Wizard.
/// Magecraft loot via `magecraft_loot()` — premium top-end value
/// engine. The body scales naturally with spell-heavy shells.
pub fn prismari_crashbinder_b154() -> CardDefinition {
    CardDefinition {
        name: "Prismari Crashbinder (b154)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Sparkglyph (b154) — {1}{R} Instant. Deal 3 damage to any
/// target — clean Prismari burn at the 2-mana slot.
pub fn prismari_sparkglyph_b154() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkglyph (b154)",
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

/// Prismari Stormbreaker (b154) — {3}{U}{R} 4/3 Elemental Wizard.
/// ETB Seq(deal 2 to any target + Draw 1). 5-mana versatile threat.
pub fn prismari_stormbreaker_b154() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Prismari Stormbreaker (b154)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
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
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
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

/// Prismari Flameseeker (b154) — {1}{U}{R} 2/2 Human Wizard. Magecraft
/// loot via `magecraft_loot()`. Compact spell-slinger value engine.
pub fn prismari_flameseeker_b154() -> CardDefinition {
    CardDefinition {
        name: "Prismari Flameseeker (b154)",
        cost: cost(&[generic(1), u(), r()]),
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
        triggered_abilities: vec![magecraft_loot()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Prismari Calligrapher (b154) — {2}{U} 2/3 Merfolk Wizard. ETB
/// Scry 2. Defensive flier-less smoothing body.
pub fn prismari_calligrapher_b154() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Prismari Calligrapher (b154)",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_scry(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}
