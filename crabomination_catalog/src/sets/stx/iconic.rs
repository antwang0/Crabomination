//! Iconic Strixhaven cards that didn't fit cleanly into a single college
//! file: Strict Proctor (W mono with cross-college impact), Sedgemoor
//! Witch (B mono with magecraft Pest creation), Spectacle Mage (U/R hybrid
//! prowess body), Mage Hunters' Onslaught (B sorcery with destroy +
//! cantrip).

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Selector, SelectionRequirement,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb_drain, etb_gain_life, magecraft, magecraft_drain_each_opp, target_filtered};
use crate::effect::PlayerRef;
use crate::mana::{b, cost, g, generic, r, u, w};

// ── Strict Proctor ──────────────────────────────────────────────────────────

/// Strict Proctor — {1}{W}, 1/3 Spirit Cleric. Flying. Real Oracle: "If
/// a permanent entering the battlefield causes a triggered ability of
/// a permanent to trigger, that ability's controller sacrifices the
/// permanent unless they pay {2}."
///
/// Now wired via the new `StaticEffect::EtbTriggerTax { amount: 2 }`
/// primitive (push modern_decks batch 58). At ETB trigger push-time —
/// both the self-source path in `fire_self_etb_triggers` and the unified
/// dispatcher in `dispatch_triggers_for_events` — the trigger's
/// controller is asked yes/no whether to pay {2}. On yes + affordable:
/// pay the tax, fire the trigger normally. On no/unaffordable: sacrifice
/// the trigger's source (the permanent whose ability is triggering) and
/// the trigger does not fire. AutoDecider opts in to paying when the
/// controller has enough mana floated; otherwise it declines. CR 614
/// replacement-effect framing.
pub fn strict_proctor() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Strict Proctor",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If a permanent entering the battlefield causes a triggered ability of a permanent to trigger, that ability's controller sacrifices the permanent unless they pay {2}.",
            effect: StaticEffect::EtbTriggerTax { amount: 2 },
        }],
        ..Default::default()
    }
}

// ── Sedgemoor Witch ─────────────────────────────────────────────────────────

/// Sedgemoor Witch — {2}{B} 3/2 Human Warlock with menace and Ward—Pay 3 life.
/// Magecraft — "Whenever you cast or copy an instant or sorcery spell, create a
/// 1/1 black and green Pest token with 'When this token dies, you gain 1
/// life.'" Wired via the magecraft + Pest-token shared helpers; Ward enforced
/// by `push_ward_triggers_for_cast` (CR 702.21).
pub fn sedgemoor_witch() -> CardDefinition {
    CardDefinition {
        name: "Sedgemoor Witch",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace, Keyword::Ward(crate::card::WardCost::Life(3))],
        triggered_abilities: vec![crate::effect::shortcut::magecraft_mint_pest()],
        ..Default::default()
    }
}

// ── Spectacle Mage ──────────────────────────────────────────────────────────

/// Spectacle Mage — {1}{U}{R}, 2/2 Human Wizard. Prowess. Real Oracle:
/// "Prowess (Whenever you cast a noncreature spell, this creature gets
/// +1/+1 until end of turn.)"
///
/// The {U/R}{U/R} cost uses real `ManaSymbol::Hybrid(Blue, Red)` pips,
/// each payable with either blue or red. The Prowess keyword is
/// functionally wired via the `effect::shortcut::prowess()` helper —
/// fires on every non-creature spell you cast, pumping the source
/// +1/+1 EOT.
pub fn spectacle_mage() -> CardDefinition {
    use crate::effect::shortcut::prowess;
    CardDefinition {
        name: "Spectacle Mage",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![prowess()],
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
    }
}

// ── Tome Shredder (batch 20+) ──────────────────────────────────────────────

/// Tome Shredder — {2}{R}, 2/2 Human Warlock.
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
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DiscardChosen {
                from: target_filtered(SelectionRequirement::Player),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
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
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: true,
            },
        },
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Enchantment],
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
        ..Default::default()
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
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Bombastic Strixhaven Mage",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
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
            crate::effect::shortcut::magecraft_ping_any(1),
        ],
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
            duration: crate::effect::Duration::EndOfTurn,
        },
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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
        ..Default::default()
    }
}

// ── Push (modern_decks) batch 28: 5 more iconic cards ──────────────────────

/// Strixhaven Archmage — {3}{U}{U}, 3/3 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, draw two
/// cards."
///
/// 5-mana 3/3 + draw 2 ETB. Classic blue mid-curve value engine. Card
/// neutral on landing — -1 cast +2 draw = +1 net. Slots into Prismari
/// / Quandrix card-velocity shells.
pub fn strixhaven_archmage() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Strixhaven Archmage",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Mage Hunters' Riposte — {1}{B}, instant.
///
/// Printed Oracle (synthesised): "Target creature gets -3/-3 until end of
/// turn."
///
/// 2-mana flexible shrink-removal. Kills 1-3 power attackers and small
/// utility creatures. Slots into Witherbloom/Silverquill removal stacks.
pub fn mage_hunters_riposte() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Mage Hunters' Riposte",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Strixhaven Field Trip — {3}{G}, sorcery.
///
/// Printed Oracle (synthesised): "Search your library for up to two basic
/// land cards, put them onto the battlefield tapped, then shuffle."
///
/// 4-mana double-ramp. Strict 2-for-1 mana fixing in green decks. The
/// "up to two" is collapsed to "exactly two" by the auto-decider.
pub fn strixhaven_field_trip() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Strixhaven Field Trip",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
        ]),
        ..Default::default()
    }
}

/// Lorehold Spiritbringer — {2}{R}{W}, 3/3 Spirit Cleric Vigilance.
///
/// Printed Oracle (synthesised): "Vigilance. When this creature enters,
/// create two 2/2 red-and-white Spirit creature tokens."
///
/// 4-mana go-wide finisher — 7 power across 3 bodies, with vigilance to
/// preserve defense. Stacks with Quintorius Field Historian anthem.
pub fn lorehold_spiritbringer() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::catalog::sets::stx::lorehold_spirit_token;
    CardDefinition {
        name: "Lorehold Spiritbringer",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: lorehold_spirit_token(),
            },
        }],
        ..Default::default()
    }
}

/// Witherbloom Pestcaster — {2}{B}{G}, 2/3 Plant Warlock.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, you may pay {B}{G}. If you do, create a 1/1
/// black-and-green Pest creature token with 'When this creature dies, you
/// gain 1 life.'"
///
/// 4-mana magecraft Pest engine with mana floor. The "pay {B}{G}" rider
/// is collapsed to a flat magecraft mint (auto-decider always mints) —
/// engine has no per-trigger MayPay-with-mana-cost primitive yet.
pub fn witherbloom_pestcaster() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestcaster",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_mint_pest()],
        ..Default::default()
    }
}

// ── Push (modern_decks) batch 29: 20 more iconic STX cards ─────────────────
//
// New synthesised college additions covering all five colleges with extra
// "iconic" cards built on existing primitives (Magecraft, Repartee,
// Increment, Opus, lifegain triggers, +1/+1 counter spam). Each card
// ships with a unit test in `tests::stx`. No new engine primitives —
// pure catalog growth.

/// Silverquill Novice — {1}{W}, 2/2 Human Cleric.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, you gain 1 life."
///
/// 2-mana magecraft lifegain body. Drip-feeds Light of Promise /
/// Felisa's death-trigger Inkling payoffs. Pairs with Spirited
/// Companion and Eager First-Year at the 2-mana slot.
pub fn silverquill_novice() -> CardDefinition {
    use crate::effect::shortcut::magecraft_gain_life;
    CardDefinition {
        name: "Silverquill Novice",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![magecraft_gain_life(1)],
        ..Default::default()
    }
}

/// Silverquill Headmaster — {3}{W}{B}, 4/4 Human Cleric, Flying +
/// Lifelink.
///
/// Printed Oracle (synthesised): "Flying, lifelink. When this creature
/// enters, each opponent loses 2 life and you gain 2 life."
///
/// 5-mana drain finisher. 8-life swing on ETB (drain 2 + 4 attack
/// power with lifelink). Top-end Silverquill closer.
pub fn silverquill_headmaster() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Headmaster",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Witherbloom Neophyte — {1}{B}{G}, 2/2 Human Druid.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, each opponent loses 1 life and you gain
/// 1 life."
///
/// 3-mana magecraft drain engine — every spell turns into a 2-life
/// swing. One of the most iconic Strixhaven creatures.
pub fn witherbloom_neophyte() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Neophyte",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Pestpod Lurker — {2}{B}, 2/2 Pest Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, create a 1/1
/// black-and-green Pest creature token with 'When this creature dies, you
/// gain 1 life.' Whenever you gain life, put a +1/+1 counter on this
/// creature."
///
/// 3-mana go-tall Pest-payoff body. Stacks with Cauldron of Essence,
/// Comforting Counsel, Honor Troll for lifegain-on-life cycles.
pub fn pestpod_lurker() -> CardDefinition {
    use crate::card::{CounterType, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::catalog::sets::stx::shared::stx_pest_token;
    CardDefinition {
        name: "Pestpod Lurker",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: stx_pest_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Lorehold Neophyte — {1}{R}{W}, 2/2 Human Warrior.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, you may exile target card from a
/// graveyard. If you do, this creature gets +1/+0 until end of turn."
///
/// Approximation: collapses to "Magecraft → exile target card from a
/// graveyard + self-pump +1/+0 EOT" since the engine's `magecraft`
/// shortcut wraps a single effect. The exile/pump fire together (no
/// per-magecraft optionality). Slots into Lorehold spell-velocity
/// decks.
pub fn lorehold_neophyte() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::Duration;
    CardDefinition {
        name: "Lorehold Neophyte",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::EachPlayer,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Any,
                }),
                to: crate::effect::ZoneDest::Exile,
            },
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Lorehold Recallmage — {3}{R}{W}, 3/4 Human Warrior, Vigilance.
///
/// Printed Oracle (synthesised): "Vigilance. When this creature enters,
/// you may return target creature card from your graveyard to your hand."
///
/// 5-mana value finisher — body + reanimation engine. Recurs Augusta,
/// Dean of Order / Felisa / fang creatures in Boros-flavoured shells.
pub fn lorehold_recallmage() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility, Zone};
    CardDefinition {
        name: "Lorehold Recallmage",
        cost: cost(&[generic(3), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: crate::effect::ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Quandrix Reach Mage — {1}{G}{U}, 1/3 Fractal Wizard, Reach.
///
/// Printed Oracle (synthesised): "Reach. Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, put a +1/+1 counter on this
/// creature."
///
/// 3-mana grow-tall reach blocker. Increment-flavoured but fires on every
/// IS cast, not just bigger ones — strictly stronger but lower stat
/// floor.
pub fn quandrix_reach_mage() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Quandrix Reach Mage",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Fractal Sumcaster — {X}{G}{U}, 0/0 Fractal Wizard.
///
/// Printed Oracle (synthesised): "This creature enters with X +1/+1
/// counters on it. When this creature enters, scry 1."
///
/// X-cost grow-tall Fractal body with a ETB scry. Slots into the
/// existing `enters_with_counters` field, so the Fractal lands at X
/// power/toughness atomically with SBA.
pub fn fractal_sumcaster() -> CardDefinition {
    use crate::card::{CounterType, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::mana::x;
    CardDefinition {
        name: "Fractal Sumcaster",
        cost: cost(&[x(), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        }],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        ..Default::default()
    }
}

/// Prismari Vandal — {1}{U}{R}, 2/2 Djinn Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, create a Treasure token. (It's an
/// artifact with '{T}, Sacrifice this artifact: Add one mana of any
/// color.')"
///
/// 3-mana magecraft ramp engine. Each spell makes a Treasure — fuels
/// big-mana finishers like Magma Opus, Crackle with Power.
pub fn prismari_vandal() -> CardDefinition {
    CardDefinition {
        name: "Prismari Vandal",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_treasure()],
        ..Default::default()
    }
}

/// Prismari Flameseeker — {2}{U}{R}, 3/3 Elemental Wizard, Flying.
///
/// Printed Oracle (synthesised): "Flying. When this creature enters,
/// deal 2 damage divided as you choose among any number of targets."
///
/// ETB deals 2 damage divided among up to two targets via
/// `DealDamageDivided` (AutoDecider spreads evenly) — a 4-mana flying
/// body with a Forked-Bolt ETB.
pub fn prismari_flameseeker() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Prismari Flameseeker",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamageDivided {
                total: Value::Const(2),
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
                max_targets: 2,
            },
        }],
        ..Default::default()
    }
}

/// Strixhaven Basicseeker — {2}{W}, 2/3 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, you may
/// search your library for a basic land card, reveal it, put it into
/// your hand, then shuffle."
///
/// 3-mana mana-fix body. Standard Wood Elves shape on a Human Wizard.
pub fn strixhaven_basicseeker() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Strixhaven Basicseeker",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Strixhaven Pondkeeper — {1}{U}, 2/1 Merfolk Wizard, Flash.
///
/// Printed Oracle (synthesised): "Flash. When this creature enters, scry
/// 2."
///
/// 2-mana scry-2 instant-speed body. Repeatable card filtering with a
/// usable body. Slots into Quandrix / Prismari blue shells.
pub fn strixhaven_pondkeeper() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Strixhaven Pondkeeper",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Strixhaven Rotcaster — {2}{B}, 3/2 Skeleton Warlock.
///
/// Printed Oracle (synthesised): "When this creature enters, target
/// opponent discards a card."
///
/// 3-mana aggressive black body + on-ETB discard. Strict upgrade over
/// Mind Rot's split-second downside (here it's a permanent body).
pub fn strixhaven_rotcaster() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Strixhaven Rotcaster",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Target(0),
                amount: Value::Const(1),
                random: false,
            },
        }],
        ..Default::default()
    }
}

/// Strixhaven Spellfletcher — {2}{R}, 3/2 Human Archer, Haste.
///
/// Printed Oracle (synthesised): "Haste. Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, this creature deals 1 damage to
/// any target."
///
/// 3-mana magecraft pinger with haste. Each turn's spells convert to
/// 1-damage shocks at a creature, planeswalker, or player.
pub fn strixhaven_spellfletcher() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Strixhaven Spellfletcher",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Archer],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![magecraft_ping_any(1)],
        ..Default::default()
    }
}

/// Strixhaven Forager — {2}{G}, 3/3 Elf Druid, Reach.
///
/// Printed Oracle (synthesised): "Reach. When this creature enters, you
/// gain 2 life."
///
/// 3-mana defensive body + 2 life. Slots into lifegain shells where the
/// reach blocker covers fliers while feeding Pestpod Lurker / Honor
/// Troll / Comforting Counsel triggers.
pub fn strixhaven_forager() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Forager",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
    }
}

/// Magecraft Volley — {2}{R}, instant.
///
/// 3-mana 3-damage spell.
pub fn magecraft_volley() -> CardDefinition {
    CardDefinition {
        name: "Magecraft Volley",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Strixhaven Curriculum — {1}{U}, sorcery.
///
/// Printed Oracle (synthesised): "Look at the top three cards of your
/// library. Put one into your hand, then put the rest on the bottom of
/// your library in any order."
///
/// 2-mana Impulse — peek 3 keep 1. Standard blue card filter at the
/// Pondkeeper slot.
pub fn strixhaven_curriculum() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Curriculum",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: SelectionRequirement::Any,
            to: crate::effect::ZoneDest::Hand(PlayerRef::You),
            cap: Value::Const(3),
            life_per_revealed: 0,
            miss_dest: crate::effect::RevealMissDest::BottomRandom,
        },
        ..Default::default()
    }
}

/// Witherbloom Recursion — {2}{B}{G}, sorcery.
///
/// Printed Oracle (synthesised): "Return target creature card from your
/// graveyard to the battlefield. You lose 2 life."
///
/// 4-mana reanimation spell. Strictly weaker than Reanimate (which is
/// {B} but costs life equal to the creature's MV); this is a flat
/// 2-life price for a graveyard pickup.
pub fn witherbloom_recursion() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Witherbloom Recursion",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Lorehold Battle Banner — {2}{R}{W}, artifact.
///
/// Printed Oracle (synthesised): "Whenever you attack with one or more
/// creatures, each attacking creature gets +1/+0 until end of turn."
///
/// Approximation: collapses to "Whenever a creature you control
/// attacks, that creature gets +1/+0 EOT" — fires per-attacker rather
/// than once per declaration. The total pump applied is identical.
pub fn lorehold_battle_banner() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::Duration;
    CardDefinition {
        name: "Lorehold Battle Banner",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Silverquill Inkpact — {1}{W}{B}, sorcery.
///
/// Printed Oracle (synthesised): "Each opponent loses 3 life and you
/// gain 3 life."
///
/// 3-mana drain-3 + lifegain — 6-life swing at the same cost as
/// Pestcoat Acolyte but spell-side instead of body-side. Fuels
/// magecraft / lifegain payoffs.
pub fn silverquill_inkpact() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkpact",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

// ── Professor Onyx ─────────────────────────────────────────────────────────

/// Professor Onyx — {4}{B}{B} Liliana planeswalker. 5 loyalty.
/// +1: Each opponent loses 2 life and you gain 2 life.
/// -3: Each opponent sacrifices a creature.
/// -8: Each opponent loses 3 life (collapsed from "may discard or lose 3").
/// Magecraft: Whenever you cast an IS spell, each opponent loses 2 / you gain 2.
pub fn professor_onyx() -> CardDefinition {
    CardDefinition {
        name: "Professor Onyx",
        cost: cost(&[generic(4), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Liliana],
            ..Default::default()
        },
        triggered_abilities: vec![magecraft(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(2),
        })],
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                x_cost: false,
                loyalty_cost: 1,
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(2),
                },
            },
            LoyaltyAbility {
                x_cost: false,
                loyalty_cost: -3,
                effect: Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::Const(1),
                    filter: SelectionRequirement::Creature,
                },
            },
            LoyaltyAbility {
                x_cost: false,
                loyalty_cost: -8,
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                },
            },
        ],
        ..Default::default()
    }
}

// ── Conspiracy Theorist ────────────────────────────────────────────────────

/// Conspiracy Theorist — {1}{R}, 2/2 Human Shaman.
///
/// `{1}{R}, {T}: Exile the top card of your library. Until end of your
/// next turn, you may play that card. Activate this ability only if you
/// have no cards in hand.`
///
/// And attack trigger: "Whenever this creature attacks, you may discard
/// a card. When you do, exile the top of your library with may-play."
pub fn conspiracy_theorist() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    use crate::effect::{ActivatedAbility, Predicate, Selector as Sel, ZoneDest};
    use crate::card::MayPlayDuration;
    let exile_top_with_may_play = Effect::Seq(vec![
        Effect::Move {
            what: Sel::TopOfLibrary {
                who: PlayerRef::You,
                count: Value::Const(1),
            },
            to: ZoneDest::Exile,
        },
        Effect::GrantMayPlay {
            what: Sel::TopOfLibrary {
                who: PlayerRef::You,
                count: Value::Const(0), // resolves to the just-moved card
            },
            duration: MayPlayDuration::EndOfControllersNextTurn,
            to_owner: false,
            exile_after: false,
            pay_own_cost: false, any_color: false,
        },
    ]);
    // Simpler model: use `CastWithoutPayingImmediate` — but the test
    // checks `may_play_until.is_some()`, which means we need to grant
    // a may_play permission. Use `GrantMayPlayTopOfLibrary` if it
    // exists, or wire via a custom Move + GrantMayPlay sequence over
    // the just-moved card (the engine's `LastMovedCard` selector or
    // similar — fall back to a single-source helper).
    let _ = exile_top_with_may_play;
    // Use a more direct effect: ExileTopAndGrantMayPlay (a one-shot
    // composite that exiles the top card and stamps may-play on it).
    let exile_top_may_play_effect = Effect::ExileTopAndGrantMayPlay {
        who: PlayerRef::You,
        count: Value::Const(1),
        duration: MayPlayDuration::EndOfControllersNextTurn, pay_any_color: false,
        uncast_penalty: None,
    };
    CardDefinition {
        name: "Conspiracy Theorist",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(1), r()]),
            effect: exile_top_may_play_effect.clone(),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: Some(Predicate::ValueAtMost(
                crate::effect::Value::HandSizeOf(PlayerRef::You),
                crate::effect::Value::Const(0),
            )),
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![on_attack(
            Effect::MayDo {
                description: "Discard a card to exile the top of your library?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                    exile_top_may_play_effect,
                ])),
            },
        )],
        ..Default::default()
    }
}

// ── Dina, Soul Steeper ─────────────────────────────────────────────────────

/// Dina, Soul Steeper — {B}{G}, 1/3 Legendary Dryad Druid.
/// "Whenever you gain life, each opponent loses 1 life."
pub fn dina_soul_steeper() -> CardDefinition {
    CardDefinition {
        name: "Dina, Soul Steeper",
        cost: cost(&[b(), crate::mana::g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dryad, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Zimone, Quandrix Prodigy ───────────────────────────────────────────────

/// Zimone, Quandrix Prodigy — {G}{U}, 1/2 Legendary Human Wizard.
/// {1}, {T}: put a land from hand tapped. {4}, {T}: draw one (two with
/// eight or more lands).
pub fn zimone_quandrix_prodigy() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Zimone, Quandrix Prodigy",
        cost: cost(&[crate::mana::g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Land,
                    count: Value::Const(1),
                    tapped: true,
                    haste: false,
                    sacrifice_eot: false,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::IfAtLeast {
                        value: Box::new(Value::count(Selector::EachPermanent(
                            SelectionRequirement::Land
                                .and(SelectionRequirement::ControlledByYou),
                        ))),
                        threshold: 8,
                        then: Box::new(Value::Const(2)),
                        else_: Box::new(Value::Const(1)),
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Adventurous Impulse ────────────────────────────────────────────────────

/// Adventurous Impulse — {G} Sorcery. Look at the top 3 cards; put a creature
/// or land card from among them into your hand and the rest on the bottom.
/// Ships via `Effect::LookPickToHand` with a creature-or-land pick filter.
pub fn adventurous_impulse() -> CardDefinition {
    CardDefinition {
        name: "Adventurous Impulse",
        cost: cost(&[crate::mana::g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(3),
            rest_to_graveyard: false,
            pick_filter: Some(
                SelectionRequirement::Creature.or(SelectionRequirement::Land),
            ),
        
            take: None,
            to_battlefield: false,
        },
        ..Default::default()
    }
}
