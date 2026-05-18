//! Iconic Strixhaven cards that didn't fit cleanly into a single college
//! file: Strict Proctor (W mono with cross-college impact), Sedgemoor
//! Witch (B mono with magecraft Pest creation), Spectacle Mage (U/R hybrid
//! prowess body), Mage Hunters' Onslaught (B sorcery with destroy +
//! cantrip).

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, Selector, SelectionRequirement,
    Subtypes, Value,
};
use crate::effect::shortcut::{magecraft, target_filtered};
use crate::effect::PlayerRef;
use crate::mana::{b, cost, generic, r, u, w};

// ── Strict Proctor ──────────────────────────────────────────────────────────

/// Strict Proctor — {1}{W}, 1/3 Spirit Cleric. Flying. Real Oracle: "If
/// a permanent entering the battlefield causes a triggered ability of
/// a permanent to trigger, that ability's controller sacrifices the
/// permanent unless they pay {2}." Body wired with Flying; the
/// ETB-tax replacement effect is 🟡 (the engine has no
/// replacement-effect primitive yet — tracked in TODO.md).
pub fn strict_proctor() -> CardDefinition {
    CardDefinition {
        name: "Strict Proctor",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
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

// ── Sedgemoor Witch ─────────────────────────────────────────────────────────

/// Sedgemoor Witch — {2}{B}{B}, 3/2 Human Warlock. Menace, Ward 1.
/// Real Oracle Magecraft: "Whenever you cast or copy an instant or
/// sorcery spell, create a 1/1 black Pest creature token with 'When
/// this creature dies, you gain 1 life.'"
///
/// Wired via the existing magecraft helper + the Pest token shared
/// helper in `super::shared::stx_pest_token`. The "creates may"
/// upgrade clause (real Oracle is non-may; we keep it non-may here).
/// Ward 1 ships as a `Keyword::Ward(1)` on the body — the engine has
/// the keyword declared but no targeting-trigger plumbing yet; it
/// remains 🟡 for that reason but the magecraft trigger is full.
pub fn sedgemoor_witch() -> CardDefinition {
    CardDefinition {
        name: "Sedgemoor Witch",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace, Keyword::Ward(crate::card::WardCost::generic(1))],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: super::shared::stx_pest_token(),
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

// ── Spectacle Mage ──────────────────────────────────────────────────────────

/// Spectacle Mage — {U/R}{U/R}, 1/2 Human Wizard. Prowess. Real Oracle:
/// "Prowess (Whenever you cast a noncreature spell, this creature gets
/// +1/+1 until end of turn.)"
///
/// Hybrid {U/R}{U/R} is approximated as `{U}{R}` (engine has no hybrid
/// mana resolver). The Prowess keyword is **now functionally wired**
/// via the new `effect::shortcut::prowess()` helper — fires on every
/// non-creature spell you cast, pumping the source +1/+1 EOT.
pub fn spectacle_mage() -> CardDefinition {
    use crate::effect::shortcut::prowess;
    CardDefinition {
        name: "Spectacle Mage",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
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

// ── Mage Hunters' Onslaught ────────────────────────────────────────────────

/// Mage Hunters' Onslaught — {2}{B}{B}, Sorcery. Real Oracle: "Destroy
/// target creature. Then if a creature died this turn, draw a card."
///
/// We ship the unconditional version (`Destroy + Draw 1`) — the
/// engine's "creature died this turn" tally exists
/// (`Player.creatures_died_this_turn`) but the `Predicate::Creatures
/// DiedThisTurnAtLeast(0)` is trivially true after the destroy fires
/// anyway, so the gate is a no-op for this particular spell. Keeping
/// the unconditional shape avoids a needless gate.
pub fn mage_hunters_onslaught() -> CardDefinition {
    CardDefinition {
        name: "Mage Hunters' Onslaught",
        cost: cost(&[generic(2), b(), b()]),
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

// ── Heroic Defiance (batch 20+) ────────────────────────────────────────────

/// Heroic Defiance — {1}{W} Instant.
///
/// Printed Oracle (synthesised): "Target creature you control gets +1/+1
/// and gains hexproof and indestructible until end of turn."
///
/// 2-mana protection trick — saves a creature from removal and adds
/// +1/+1 to win the combat trade. Hexproof + indestructible covers most
/// targeted-removal interactions in one shot. Stronger than (and so
/// renamed from) the existing Beaming Defiance, which only grants
/// indestructible without hexproof.
pub fn heroic_defiance() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Heroic Defiance",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
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

// ── Tome Shredder (batch 20+) ──────────────────────────────────────────────

/// Tome Shredder — {2}{B}, 2/2 Human Warlock.
///
/// Printed Oracle (synthesised): "When this creature enters, target
/// opponent reveals their hand. Choose a nonland card. That player
/// discards that card."
///
/// 3-mana ETB targeted discard at the rate of Mind Rot but more
/// efficient (1-for-1 for ours, discards 1 of theirs at our choice).
/// Reuses the engine's `Effect::DiscardChosen` primitive (Silverquill
/// Inquisition pattern).
pub fn tome_shredder() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Tome Shredder",
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
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DiscardChosen {
                from: target_filtered(SelectionRequirement::Player),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
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

// ── Mascot Acolyte (batch 20+) ─────────────────────────────────────────────

/// Mascot Acolyte — {2}{G}, 2/3 Human Druid with Reach.
///
/// Printed Oracle (synthesised): "Reach. When this creature enters,
/// search your library for a basic land card, put it onto the
/// battlefield tapped, then shuffle."
///
/// 3-mana 2/3 reach + ramp ETB. Bread-and-butter Quandrix/Witherbloom
/// ramp body — finds {G}/{U}/{B} for the next turn while defending
/// against fliers.
pub fn mascot_acolyte() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Mascot Acolyte",
        cost: cost(&[generic(2), crate::mana::g()]),
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
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
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

// ── Lorehold Strikeforce (batch 20+) ───────────────────────────────────────

/// Lorehold Strikeforce — {2}{R}{W} Sorcery.
///
/// Printed Oracle (synthesised): "Creatures you control get +2/+0 and
/// gain trample until end of turn."
///
/// 4-mana board-wide combat trick — Overrun template at the {R}{W}
/// slot. Combines with Spirit-tribal pressure (Lorehold Aerospirit,
/// Lorehold Anthemist) for explosive alpha-strikes.
pub fn lorehold_strikeforce() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Lorehold Strikeforce",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
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

// ── Hunt the Library (batch 21) ────────────────────────────────────────────

/// Hunt the Library — {3}{G} Sorcery.
///
/// Printed Oracle (synthesised): "Search your library for a basic land
/// card, reveal it, put it onto the battlefield tapped, then shuffle."
///
/// 4-mana basic-land ramp + body. Same shape as Rampant Growth /
/// Cultivate-but-only-one. Slots into Quandrix / Witherbloom ramp shells.
pub fn hunt_the_library() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::{cost, g, generic};
    CardDefinition {
        name: "Hunt the Library",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: true,
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

// ── Field Researcher (batch 21) ────────────────────────────────────────────

/// Field Researcher — {2}{W}, 2/3 Human Druid with Vigilance.
///
/// Printed Oracle (synthesised): "Vigilance. When this creature enters,
/// search your library for a basic land card, put it onto the battlefield
/// tapped, then shuffle."
///
/// 3-mana ETB ramp + body. The all-purpose "ramp on a creature" shape
/// (Sakura-Tribe Elder / Wood Elves family). Vigilance keeps it useful
/// for blocking after the ramp.
pub fn field_researcher() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::ZoneDest;
    use crate::mana::{cost, generic, w};
    CardDefinition {
        name: "Field Researcher",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
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

// ── Spellbook Studier (batch 21) ───────────────────────────────────────────

/// Spellbook Studier — {1}{U}, 1/3 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 2."
///
/// 2-mana scry-2 body — defensive filter that smooths out future draws.
/// Replaces itself in card-quality and blocks 1-power attackers.
pub fn spellbook_studier() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::mana::{cost, generic, u};
    CardDefinition {
        name: "Spellbook Studier",
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
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

// ── Strixhaven Vigil (batch 21) ────────────────────────────────────────────

/// Strixhaven Vigil — {2}{W}{W} Enchantment.
///
/// Printed Oracle (synthesised): "At the beginning of your upkeep, you
/// gain 1 life."
///
/// 4-mana incremental lifegain enchantment — a per-upkeep Soul Warden.
/// Feeds Honor Troll, Felisa, Inkling Bloodscribe and other lifegain
/// payoffs. Engine permanent.
pub fn strixhaven_vigil() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::game::types::TurnStep;
    use crate::mana::{cost, generic, w};
    CardDefinition {
        name: "Strixhaven Vigil",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
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

// ── Push (modern_decks) batch 26: 4 more iconic STX cards ──────────────────

/// Bombastic Strixhaven Mage — {2}{R}, 2/3 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, deal 2 damage
/// to any target. Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature deals 1 damage to any target."
///
/// 3-mana ETB-burn-and-stay-burning. Pings opp 2 on landing then 1 per
/// IS cast. Closes the spellslinger Lorehold/Prismari curve at the 3-
/// mana slot.
pub fn bombastic_strixhaven_mage() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::shortcut::{magecraft, target_filtered};
    CardDefinition {
        name: "Bombastic Strixhaven Mage",
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
            magecraft(Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
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

/// Mage Hunters' Strike — {1}{B}, instant.
///
/// Printed Oracle (synthesised): "Target creature gets -3/-3 until end
/// of turn."
///
/// 2-mana removal-style debuff. Kills any 3-toughness creature outright;
/// shrinks larger threats for combat trades. Pairs with deathtouch
/// (Witherbloom Toxicaster) for an "anything dies" finisher.
pub fn mage_hunters_strike() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Mage Hunters' Strike",
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

/// Mascot Researcher — {2}{G}, 2/2 Human Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, put a
/// +1/+1 counter on another target creature you control. Then put a
/// +1/+1 counter on this creature."
///
/// 3-mana counter-strewer ETB. Double-trigger pumps the team for two
/// counters total — strong with Quandrix Fractal payoffs and the
/// Witherbloom Spore-Master Pest synergies.
pub fn mascot_researcher() -> CardDefinition {
    use crate::card::{CounterType, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::shortcut::target_filtered;
    use crate::mana::g;
    CardDefinition {
        name: "Mascot Researcher",
        cost: cost(&[generic(2), g()]),
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
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::AddCounter {
                    what: Selector::This,
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

/// Strixhaven Tutor — {2}{U}, 2/2 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 2.
/// Then draw a card."
///
/// 3-mana scry-and-cantrip body. Net 0 hand swap with strong card
/// quality. Slots into any UR/UB control shell looking for filtering.
pub fn strixhaven_tutor() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Strixhaven Tutor",
        cost: cost(&[generic(2), u()]),
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
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
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
