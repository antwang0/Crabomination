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
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{magecraft, magecraft_self_pump, target_filtered};
use crate::effect::{Duration, PlayerRef};
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
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
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
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
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
    }
}
