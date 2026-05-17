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
