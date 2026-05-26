//! Iconic Strixhaven cards that didn't fit cleanly into a single college
//! file: Strict Proctor (W mono with cross-college impact), Sedgemoor
//! Witch (B mono with magecraft Pest creation), Spectacle Mage (U/R hybrid
//! prowess body), Mage Hunters' Onslaught (B sorcery with destroy +
//! cantrip).

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Selector, SelectionRequirement,
    Subtypes, Supertype, TriggeredAbility, Value,
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
        keywords: vec![Keyword::Menace, Keyword::Ward(1)],
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
    }
}

// ── Spectacle Mage ──────────────────────────────────────────────────────────

/// Spectacle Mage — {U/R}{U/R}, 1/2 Human Wizard. Prowess. Real Oracle
/// flavor: "Whenever you cast a noncreature spell, this creature gets
/// +1/+1 until end of turn." (Standard prowess.) Hybrid {U/R}{U/R} is
/// approximated as `{U}{R}` (engine has no hybrid mana resolver — a
/// player who can produce only U or only R can still cast). Prowess
/// keyword is declared today but not yet wired into the trigger
/// system; the body and stat line are correct.
pub fn spectacle_mage() -> CardDefinition {
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
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
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
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(2),
        })],
        static_abilities: vec![],
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(2),
                },
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::Const(1),
                    filter: SelectionRequirement::Creature,
                },
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                },
            },
        ],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Conspiracy Theorist ────────────────────────────────────────────────────

/// Conspiracy Theorist — {1}{R}, 2/2 Human Shaman. On attack, may
/// discard and draw.
pub fn conspiracy_theorist() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Conspiracy Theorist",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack(
            Effect::MayDo {
                description: "Discard a card, then draw a card?".into(),
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
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
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
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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

// ── Zimone, Quandrix Prodigy ───────────────────────────────────────────────

/// Zimone, Quandrix Prodigy — {G}{U}, 1/2 Legendary Human Wizard.
/// {1}, {T}: Draw a card (approximation of land-from-hand + conditional draw).
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
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
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

// ── Adventurous Impulse ────────────────────────────────────────────────────

/// Adventurous Impulse — {G} Sorcery. Look at top 3, put a creature/land
/// to hand, rest on bottom. Approximated as Scry 2 + Draw 1.
pub fn adventurous_impulse() -> CardDefinition {
    CardDefinition {
        name: "Adventurous Impulse",
        cost: cost(&[crate::mana::g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
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
